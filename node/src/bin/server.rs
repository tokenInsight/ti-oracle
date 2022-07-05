use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use env_logger::{Builder, Env};
use ethers::prelude::U256;
use futures::channel::mpsc::channel;
use futures::SinkExt;
use libp2p::Multiaddr;
use log::{info, warn};
use std::env;
use std::error::Error;
use std::time::Duration;
use ti_node::chains;
use ti_node::fetcher::aggregator;
use ti_node::flags;
use ti_node::processor::gossip;
use ti_node::processor::gossip::ValidateRequest;
use ti_node::processor::swarm;
use ti_node::processor::utils;

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
    for peer_node in &cfg.peers {
        if peer_node.len() == 0 {
            continue;
        }
        let address: Multiaddr = peer_node.parse().expect("User to provide valid address.");
        match swarm.dial(address.clone()) {
            Ok(_) => info!("Dialed {:?}", address),
            Err(e) => warn!("Dial {:?} failed: {:?}", address, e),
        };
    }
    let (mut sender, receiver) = channel::<ValidateRequest>(128);
    let mut p2p_processor = gossip::new(swarm, topic, receiver);
    tokio::task::spawn(async move {
        p2p_processor.process_p2p_message().await;
    });
    let mut private_key = cfg.private_key.clone();
    if private_key.starts_with("$") {
        let var_name = &private_key[1..private_key.len()];
        private_key = env::var(var_name).expect("$NODE_PIVATE_KEY not set");
    }
    let oracle_stub = chains::eth::new(
        private_key,
        cfg.eth_rpc_url.clone(),
        cfg.contract_address.clone(),
    )
    .await?;
    let agg = aggregator::new(cfg.mappings.clone());
    loop {
        match oracle_stub.is_my_turn().call().await {
            Ok(is_my_turn) => {
                info!("check if it is my turn to feed? {}", is_my_turn);
                if is_my_turn {
                    let price_result = agg.get_price().await;
                    match price_result {
                        Ok(weighted_price) => {
                            collect_signatures(&oracle_stub, &cfg, &mut sender, weighted_price)
                                .await;
                        }
                        Err(err) => warn!("{}", err),
                    }
                }
            }
            Err(err) => {
                warn!("call contract error: {}", err);
            }
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

async fn collect_signatures(
    oracle_stub: &chains::eth::TIOracle<
        ethers::prelude::SignerMiddleware<
            ethers::prelude::Provider<ethers::prelude::Http>,
            ethers::prelude::Wallet<ethers::prelude::k256::ecdsa::SigningKey>,
        >,
    >,
    cfg: &flags::Config,
    sender: &mut futures::channel::mpsc::Sender<ValidateRequest>,
    weighted_price: u128,
) {
    let last_round: U256 = oracle_stub.last_round().call().await.unwrap();
    let valid_request = gossip::ValidateRequest {
        coin: cfg.coin_name.clone(),
        round: last_round.as_u64(),
        timestamp: utils::timestamp(),
        price: weighted_price.to_string(), //TODO fetch from api of market
    };
    sender.send(valid_request).await.unwrap();
}
