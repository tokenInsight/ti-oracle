use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use env_logger::{Builder, Env};
use libp2p::Multiaddr;
use std::error::Error;
use std::time::Duration;
use ti_node::fetcher;
use ti_node::flags;
use ti_node::processor::gossip;
use ti_node::processor::swarm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts = app_from_crate!()
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Configuration file path")
                .takes_value(true)
                .default_value("./config/node.yaml"),
        )
        .arg(
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
        .listen_on(cfg.listen_address.clone().parse().unwrap())
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
    tokio::task::spawn(async move {
        gossip::process_p2p_message(swarm, topic).await;
    });
    let eth_stub =
        fetcher::eth::new(cfg.private_key, cfg.eth_rpc_url, cfg.contract_address).await?;
    loop {
        match eth_stub.is_my_turn().call().await {
            Ok(is_my_turn) => {
                println!("is my turn to feed? {}", is_my_turn);
            }
            Err(err) => {
                println!("call contract error: {}", err);
            }
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
