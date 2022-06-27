use async_std::io;
use env_logger::{Builder, Env};
use futures::{prelude::*, select};
use libp2p::gossipsub::GossipsubEvent;
use libp2p::{swarm::SwarmEvent, Multiaddr};
use std::error::Error;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use ti_node::flags;
use ti_node::processor::swarm;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts = app_from_crate!()
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Configuration file path")
                .takes_value(true)
                .default_value("./config/node.yaml"),
        ).arg(
            clap::Arg::with_name("peers")
                .short("p")
                .long("peers")
                .help("set peers of this node")
                .takes_value(true)
                .default_value(""),
        )
        .get_matches();
    let peers = opts.value_of("peers").unwrap().split(",");
    let mut cfg = flags::Config::new(opts.value_of("config").unwrap())?;
    for peer_node in peers {
        cfg.peers.push(peer_node.to_string());
    }
    Builder::from_env(Env::default().default_filter_or(cfg.log_level.clone())).init();
    let (topic, mut swarm) = swarm::make_swarm(&cfg).await?;
    // Listen on all interfaces and whatever port the OS assigns
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();

    // Reach out to peers
    for peer_node in cfg.peers {
        if peer_node.len() == 0 {
            continue;
        }
        let address: Multiaddr = peer_node.parse().expect("User to provide valid address.");
        match swarm.dial(address.clone()) {
            Ok(_) => println!("Dialed {:?}", address),
            Err(e) => println!("Dial {:?} failed: {:?}", address, e),
        };
    }

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    // Kick it off
    loop {
        select! {
            line = stdin.select_next_some() => {
                if let Err(e) = swarm
                    .behaviour_mut()
                    .publish(topic.clone(), line.expect("Stdin not to close").as_bytes())
                {
                    println!("Publish error: {:?}", e);
                }
            },
            event = swarm.select_next_some() => match event {
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
