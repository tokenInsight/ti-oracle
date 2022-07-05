use crate::chains::eth;
use async_std::io;
use futures::channel::mpsc::Receiver;
use futures::{prelude::*, select};
use libp2p::gossipsub::error::PublishError;
use libp2p::gossipsub::GossipsubEvent;
use libp2p::gossipsub::IdentTopic;
use libp2p::gossipsub::MessageId;
use libp2p::swarm::SwarmEvent;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidateRequest {
    pub coin: String,
    pub price: String,
    pub round: u64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidateResponse {
    pub coin: String,
    pub price: String,
    pub round: u64,
    pub sig: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandMessage {
    VReq(ValidateRequest),
    VResp(ValidateResponse),
}
pub struct P2PMessageProcessor {
    swarm: libp2p::Swarm<libp2p::gossipsub::Gossipsub>,
    topic: IdentTopic,
    recv: Receiver<ValidateRequest>,
}

pub fn new(
    swarm: libp2p::Swarm<libp2p::gossipsub::Gossipsub>,
    topic: IdentTopic,
    recv: Receiver<ValidateRequest>,
) -> P2PMessageProcessor {
    P2PMessageProcessor {
        swarm: swarm,
        topic: topic,
        recv: recv,
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
    pub async fn process_p2p_message(&mut self, private_key: String) -> ! {
        // for debug usage
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        // Kick it off
        loop {
            select! {
                valid_req = self.recv.select_next_some() => {
                    let cmd_req = CommandMessage::VReq(valid_req);
                    let data = serde_json::to_string(&cmd_req).unwrap();
                    debug!("local command {:}", data);
                    if let Err(e) = self.publish_txt(data) {
                        warn!("Publish feed request error:{:?}", e);
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
                                    self.sign_and_sendresponse(valid_req, private_key.clone());
                                },
                                CommandMessage::VResp(valid_resps) => {
                                    info!("validate price response {:?}", valid_resps);
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

    fn sign_and_sendresponse(&mut self, valid_req: ValidateRequest, private_key: String) {
        debug!("validate price request {:?}", valid_req);
        let price = valid_req.price.parse::<u128>().unwrap();
        let sig: String = eth::sign_price_info(
            private_key.clone(),
            valid_req.coin.clone(),
            price,
            valid_req.timestamp,
        );
        debug!("sig:{}", sig);
        let sig_response = CommandMessage::VResp(ValidateResponse {
            coin: valid_req.coin,
            price: price.to_string(),
            round: valid_req.round,
            sig: sig,
            timestamp: valid_req.timestamp,
        });
        let sig_json = serde_json::to_string(&sig_response).unwrap();
        if let Err(err) = self.publish_txt(sig_json) {
            warn!("send response fail:{}", err);
        }
    }
}
