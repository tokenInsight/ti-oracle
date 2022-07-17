use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use env_logger::{Builder, Env};
use ethers::prelude::Address;
use ethers::prelude::Bytes;
use ethers::prelude::U256;
use futures::channel::mpsc::channel;
use futures::SinkExt;
use libp2p::Multiaddr;
use log::{debug, info, warn};
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use ti_node::chains;
use ti_node::chains::eth;
use ti_node::chains::eth::PeerPriceFeed;
use ti_node::fetcher::aggregator;
use ti_node::flags;
use ti_node::processor::gossip;
use ti_node::processor::gossip::LocalCommand;
use ti_node::processor::gossip::RefreshPrice;
use ti_node::processor::swarm;
use ti_node::processor::utils;
use ti_node::processor::web;
use tokio::time;
use tokio::time::timeout;

const COLLECT_RESPONSE_TIMEOUT: u64 = 5000;
const COMMIT_TX_TIMEOUT: u64 = 30000;

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
    let (mut sender, receiver) = channel::<LocalCommand>(128);
    let mut private_key = cfg.private_key.clone();
    if private_key.starts_with("$") {
        let var_name = &private_key[1..private_key.len()];
        private_key = env::var(var_name).expect("$NODE_PIVATE_KEY not set");
    }
    let oracle_stub = chains::eth::new(
        private_key.clone(),
        cfg.eth_rpc_url.clone(),
        cfg.contract_address.clone(),
    )
    .await?;
    let v_bucket = gossip::ValidationBucket::default();
    let mut p2p_processor = gossip::new(swarm, topic, receiver, Arc::clone(&v_bucket));
    tokio::task::spawn({
        let mut cfg_copy = cfg.clone();
        cfg_copy.private_key = private_key.clone();
        async move {
            p2p_processor.process_p2p_message(cfg_copy).await;
        }
    });
    let agg = aggregator::new(cfg.mappings.clone());
    let mut interval = time::interval(Duration::from_millis(cfg.feed_interval * 1000));
    let web_addr = cfg.web_address.clone();
    tokio::task::spawn(async move {
        web::start(web_addr).await;
    });
    loop {
        let price_result = agg.get_price().await;
        let weighted_price: u128;
        match price_result {
            Ok(_weighted_price) => {
                weighted_price = _weighted_price;
            }
            Err(err) => {
                tokio::time::sleep(Duration::from_millis(eth::CONTRACT_TIMEOUT)).await;
                warn!("get price from exchange fail; {}", err);
                continue;
            }
        }
        core_loop(
            &oracle_stub,
            &cfg,
            &mut sender,
            weighted_price,
            private_key.clone(),
            &v_bucket,
        )
        .await;
        info!("wait a moment to start next feeding");
        interval.tick().await;
    }
}

//core logic
async fn core_loop(
    oracle_stub: &eth::OracleStub,
    cfg: &flags::Config,
    sender: &mut futures::channel::mpsc::Sender<LocalCommand>,
    weighted_price: u128,
    private_key: String,
    v_bucket: &gossip::ValidationBucket,
) {
    let check_turn = timeout(
        Duration::from_millis(eth::CONTRACT_TIMEOUT),
        oracle_stub.is_my_turn().call(),
    )
    .await;
    match check_turn {
        Ok(check_result) => match check_result {
            Ok(is_my_turn) => {
                info!("check if it is my turn to feed? {}", is_my_turn);
                if is_my_turn {
                    let col_sig_future = collect_signatures(
                        oracle_stub,
                        cfg,
                        sender,
                        weighted_price,
                        Arc::clone(v_bucket),
                        private_key.clone(),
                    );
                    timeout(Duration::from_millis(COMMIT_TX_TIMEOUT), col_sig_future)
                        .await
                        .unwrap_or_else(|e| {
                            warn!("commit tx timeout, {}", e);
                        });
                }
                let refresh_req = RefreshPrice {
                    price: weighted_price.to_string(),
                    timestamp: utils::timestamp(),
                };
                sender
                    .send(LocalCommand::RefreshReq(refresh_req))
                    .await
                    .unwrap();
            }
            Err(err) => {
                warn!("call contract error: {}", err);
            }
        },
        Err(timeout_err) => {
            warn!("timeout err: {:?}", timeout_err);
        }
    }
}

// collect signatures from other nodes, and then commit transaction to blockchain
async fn collect_signatures(
    oracle_stub: &eth::OracleStub,
    cfg: &flags::Config,
    sender: &mut futures::channel::mpsc::Sender<LocalCommand>,
    weighted_price: u128,
    bucket: gossip::ValidationBucket,
    private_key: String,
) {
    let feed_count = match eth::get_feed_count(oracle_stub).await {
        Some(value) => value,
        None => return,
    };
    let valid_request = gossip::ValidateRequest {
        coin: cfg.coin_name.clone(),
        feed_count: feed_count.as_u64(),
        timestamp: utils::timestamp(),
        price: weighted_price.to_string(),
    };
    {
        let mut v_bucket = bucket.lock().unwrap();
        let mut gc_keys = Vec::<u64>::new();
        for (k, _) in v_bucket.iter() {
            if *k < feed_count.as_u64() {
                gc_keys.push(*k);
            }
        }
        for gc_key in gc_keys {
            v_bucket.remove(&gc_key);
        }
        //call p2p network to delivery validation request to other peers
        sender
            .send(LocalCommand::VReq(valid_request))
            .await
            .unwrap();
    }
    tokio::time::sleep(Duration::from_millis(COLLECT_RESPONSE_TIMEOUT)).await; //wait for result at most 5 seconds
    let v_bucket = bucket.lock().unwrap();
    info!("bucket size:{}", v_bucket.len());
    let collected_data = v_bucket.get(&feed_count.as_u64());
    let mut peers_price = Vec::<PeerPriceFeed>::new();
    match collected_data {
        Some(validated_response_list) => {
            info!(
                "price info signed by other nodes: {:?}",
                validated_response_list
            );
            for price_signed in validated_response_list {
                let verify_result = eth::verify_sig(
                    price_signed.sig.clone(),
                    price_signed.coin.clone(),
                    price_signed.price.parse::<u128>().unwrap_or_else(|_| 0),
                    price_signed.timestamp,
                    price_signed.address.clone(),
                );
                if !verify_result {
                    warn!("verify signature failed from {:}", price_signed.address);
                    continue;
                }
                let price_feed = PeerPriceFeed {
                    peer_address: Address::from_str(price_signed.address.as_str()).unwrap(),
                    sig: Bytes::from_str(&price_signed.sig.as_str()).unwrap(),
                    price: U256::from(price_signed.price.parse::<u128>().unwrap()),
                    timestamp: U256::from(price_signed.timestamp),
                };
                let allowed = oracle_stub
                    .query_node(price_feed.peer_address)
                    .call()
                    .await
                    .unwrap_or_else(|err| {
                        warn!("query node err: {:?}", err);
                        false
                    });
                if !allowed {
                    warn!(
                        "price report from unexpected node:{:?}",
                        price_feed.peer_address
                    );
                    continue;
                }
                peers_price.push(price_feed);
            }
        }
        None => {
            warn!("no response collected")
        }
    }
    //prepare self sign
    let ts = utils::timestamp();
    let (mysig, myaddr) = eth::sign_price_info(
        private_key.clone(),
        cfg.coin_name.clone(),
        weighted_price,
        ts,
    );
    let my_price_feed = PeerPriceFeed {
        peer_address: Address::from_str(myaddr.as_str()).unwrap(),
        sig: Bytes::from_str(mysig.as_str()).unwrap(),
        price: U256::from(weighted_price),
        timestamp: U256::from(ts),
    };
    peers_price.push(my_price_feed);
    peers_price.sort_by_key(|d| d.price);
    debug!("data will be committed: {:?}", peers_price);
    info!("commit {} items of price to blockchain", &peers_price.len());
    for price_info in &peers_price {
        info!("{} -> price: {}", price_info.peer_address, price_info.price);
    }
    match oracle_stub
        .feed_price(cfg.coin_name.clone(), peers_price)
        .gas_price(eth::from_gwei(cfg.fee_per_gas))
        .send()
        .await
    {
        Ok(pending_tx) => {
            let receipt = pending_tx.await;
            if let Err(err) = receipt {
                warn!("provider error: {}", err);
            } else {
                let result = receipt.unwrap().unwrap();
                debug!("tx receipt: {:?}", result);
                info!("transaction id: {:?}", result.transaction_hash);
                info!("gas used: {:?}", result.gas_used);
            }
        }
        Err(err) => {
            warn!("tx error: {}", err);
        }
    }
}
