use crate::flags;
use libp2p;
use libp2p::gossipsub;
use libp2p::gossipsub::GossipsubMessage;
use libp2p::gossipsub::MessageAuthenticity;
use libp2p::gossipsub::MessageId;
use libp2p::gossipsub::Topic;
use libp2p::gossipsub::ValidationMode;
use libp2p::identity;
use libp2p::PeerId;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;

pub async fn make_swarm(
    cfg: &flags::Config,
) -> Result<(gossipsub::IdentTopic, libp2p::Swarm<gossipsub::Gossipsub>), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    let transport = libp2p::development_transport(local_key.clone()).await?;
    let topic = Topic::new(cfg.coin_name.clone());
    let swarm = {
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
            // same content will be propagated.
            .build()
            .expect("Valid config");
        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                .expect("Correct configuration");
        // subscribes to our topic
        gossipsub.subscribe(&topic).unwrap();
        // build the swarm
        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };
    Ok((topic, swarm))
}
