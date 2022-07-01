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

#[derive(Serialize, Deserialize)]
pub struct FeedRequest {
    pub coin: String,
    pub round: u128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct FeedResponse {
    pub coin: String,
    pub price: u128,
    pub round: u128,
    pub sig: Vec<u8>,
    pub timestamp: u64,
}

pub struct P2PMessageProcessor {
    swarm: libp2p::Swarm<libp2p::gossipsub::Gossipsub>,
    topic: IdentTopic,
    recv: Receiver<FeedRequest>,
}

pub fn new(
    swarm: libp2p::Swarm<libp2p::gossipsub::Gossipsub>,
    topic: IdentTopic,
    recv: Receiver<FeedRequest>,
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
                req = self.recv.select_next_some() => {
                    let data = serde_json::to_string(&req).unwrap();
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
                    }) => println!(
                        "Got message: {} with id: {} from peer: {:?}",
                        String::from_utf8_lossy(&message.data),
                        id,
                        peer_id
                    ),
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    _ => {}
                }
            }
        }
    }
}
