use crate::chains::eth;
use crate::flags::Config;
use crate::processor::utils;
use async_std::io;
use futures::channel::mpsc::Receiver;
use futures::{prelude::*, select};
use libp2p::gossipsub::error::PublishError;
use libp2p::gossipsub::GossipsubEvent;
use libp2p::gossipsub::IdentTopic;
use libp2p::gossipsub::MessageId;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use super::web::SharedState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidateRequest {
    pub coin: String,
    pub price: String,
    pub feed_count: u64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidateResponse {
    pub coin: String,
    pub price: String,
    pub feed_count: u64,
    pub sig: String,
    pub timestamp: u64,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshPrice {
    pub price: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandMessage {
    VReq(ValidateRequest),
    VResp(ValidateResponse),
}

pub enum LocalCommand {
    VReq(ValidateRequest),
    RefreshReq(RefreshPrice),
}

pub type ValidationBucket = Arc<Mutex<BTreeMap<u64, Vec<ValidateResponse>>>>;
pub struct P2PMessageProcessor {
    swarm: libp2p::Swarm<libp2p::gossipsub::Gossipsub>,
    topic: IdentTopic,
    recv: Receiver<LocalCommand>,
    last_seen_price: Arc<Mutex<u128>>,
    bucket: ValidationBucket,
    s_state: SharedState,
}

pub fn new(
    swarm: libp2p::Swarm<libp2p::gossipsub::Gossipsub>,
    topic: IdentTopic,
    recv: Receiver<LocalCommand>,
    bucket: ValidationBucket,
    s_state: SharedState,
) -> P2PMessageProcessor {
    P2PMessageProcessor {
        swarm: swarm,
        topic: topic,
        recv: recv,
        last_seen_price: Arc::<Mutex<u128>>::new(Mutex::new(0)),
        bucket: bucket,
        s_state: s_state,
    }
}

impl P2PMessageProcessor {
    //helper to send string message to p2p network
    fn publish_txt(&mut self, txt: String) -> Result<MessageId, PublishError> {
        self.swarm
            .behaviour_mut()
            .publish(self.topic.clone(), txt.as_bytes())
    }

    // handle incoming events from p2p network
    pub async fn process_p2p_message(&mut self, cfg: Config) {
        // for debug usage
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        let self_eth_address = eth::pk_to_address(cfg.private_key.clone());
        // Kick it off
        loop {
            select! {
                local_cmd = self.recv.select_next_some() => {
                    match local_cmd {
                        LocalCommand::VReq(valid_req) => {
                            let cmd_req = CommandMessage::VReq(valid_req);
                            let data = serde_json::to_string(&cmd_req).unwrap();
                            debug!("local command {:}", data);
                            if let Err(e) = self.publish_txt(data) {
                                warn!("Publish feed request error:{:?}", e);
                                //try reconnect
                                for peer_node in &cfg.peers {
                                    if peer_node.len() == 0 {
                                        continue;
                                    }
                                    let address: Multiaddr = peer_node.parse().expect("User to provide valid address.");
                                    match self.swarm.dial(address.clone()) {
                                        Ok(_) => info!("Dialed {:?}", address),
                                        Err(e) => warn!("Dial {:?} failed: {:?}", address, e),
                                    };
                                }
                            }
                        },
                        LocalCommand::RefreshReq(refresh_req) => {
                            debug!("local command: {:?}", refresh_req);
                            *self.last_seen_price.lock().unwrap() = refresh_req.price.parse::<u128>().unwrap();
                        }
                    }

                },
                line = stdin.select_next_some() => {
                    if let Err(e) = self.publish_txt(line.expect("Stdin not to close"))
                    {
                        warn!("Publish debug info error: {:?}", e);
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(GossipsubEvent::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    }) => {
                        let msg_json = String::from_utf8_lossy(&message.data);
                        debug!(
                        "Got message: {} with id: {} from peer: {:?}",
                        msg_json,
                        id,
                        peer_id);
                        let cmd_result:Result<CommandMessage, serde_json::Error>= serde_json::from_str(&msg_json);
                        if cmd_result.is_ok() {
                            let cmd_result = cmd_result.unwrap();
                            match cmd_result{
                                CommandMessage::VReq(valid_req) => {
                                    self.sign_and_sendresponse(valid_req, &cfg).await;
                                },
                                CommandMessage::VResp(valid_resps) => {
                                    info!("validate price response {:?}", valid_resps);
                                    let ts = utils::timestamp();
                                    self.s_state.lock().unwrap().peers.insert(valid_resps.address.clone(), ts);
                                    let mut v_bucket = self.bucket.lock().unwrap();
                                    if !v_bucket.contains_key(&valid_resps.feed_count) {
                                        v_bucket.insert(valid_resps.feed_count, Vec::<ValidateResponse>::new());
                                    }
                                    let round_collection = v_bucket.get_mut(&valid_resps.feed_count).unwrap();
                                    let check_dup = round_collection.iter().find(|x| x.address == valid_resps.address);
                                    if check_dup.is_none() && valid_resps.address != self_eth_address {
                                        round_collection.push(valid_resps);
                                    }
                                    //info!("p2p bucket size:{}", v_bucket.len());
                                },
                            }
                        } else {
                            warn!("message error:{:?}", cmd_result.err());
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {:?}", address);
                    }
                    _ => {}
                }
            }
        }
    }

    async fn sign_and_sendresponse(&mut self, valid_req: ValidateRequest, cfg: &Config) {
        debug!("validate price request {:?}", valid_req);
        let price = valid_req.price.parse::<u128>().unwrap();
        let price_local = *self.last_seen_price.lock().unwrap();
        let diff = price_local.abs_diff(price) as f64 / price as f64;
        if diff > 0.01 {
            warn!("price diff too large: {} vs {}", price_local, price);
            return;
        } else {
            info!("price check: {} vs {}", price_local, price);
        }
        let ts_seconds = utils::timestamp() / 1000;
        let (sig, signer_address) = eth::sign_price_info(
            cfg.private_key.clone(),
            valid_req.coin.clone(),
            price_local,
            ts_seconds,
        );
        debug!("sig:{}", sig);
        let sig_response = CommandMessage::VResp(ValidateResponse {
            coin: valid_req.coin,
            price: price_local.to_string(),
            feed_count: valid_req.feed_count,
            sig: sig,
            timestamp: ts_seconds,
            address: signer_address,
        });
        let sig_json = serde_json::to_string(&sig_response).unwrap();
        if let Err(err) = self.publish_txt(sig_json) {
            warn!("send response fail:{}", err);
        }
    }
}
