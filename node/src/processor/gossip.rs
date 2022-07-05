use async_std::io;
use futures::channel::mpsc::Receiver;
use futures::{prelude::*, select};
use libp2p::gossipsub::error::PublishError;
use libp2p::gossipsub::GossipsubEvent;
use libp2p::gossipsub::IdentTopic;
use libp2p::gossipsub::MessageId;
use libp2p::swarm::SwarmEvent;
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
    pub sig: Vec<u8>,
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
    pub async fn process_p2p_message(&mut self) {
        // for debug usage
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        // Kick it off
        loop {
            select! {
                valid_req = self.recv.select_next_some() => {
                    let cmd_req = CommandMessage::VReq(valid_req);
                    let data = serde_json::to_string(&cmd_req).unwrap();
                    println!("{:}", data);
                    if let Err(e) = self.publish_txt(data) {
                        println!("Publish feed request error:{:?}", e);
                    }
                },
                line = stdin.select_next_some() => {
                    if let Err(e) = self.publish_txt(line.expect("Stdin not to close"))
                    {
                        println!("Publish debug info error: {:?}", e);
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(GossipsubEvent::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    }) => {
                        let msg_json = String::from_utf8_lossy(&message.data);
                        println!(
                        "Got message: {} with id: {} from peer: {:?}",
                        msg_json,
                        id,
                        peer_id);
                        let cmd_result:Result<CommandMessage, serde_json::Error>= serde_json::from_str(&msg_json);
                        if cmd_result.is_ok() {
                            let cmd_result = cmd_result.unwrap();
                            match cmd_result{
                                CommandMessage::VReq(valid_req) => {
                                    println!("validate price request {:?}", valid_req);
                                },
                                CommandMessage::VResp(valid_resps) => {
                                    println!("validate price response {:?}", valid_resps);
                                },
                            }
                        } else {
                            println!("message error:{:?}", cmd_result.err());
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    _ => {}
                }
            }
        }
    }
}
