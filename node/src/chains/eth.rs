use ethers::prelude::{k256::ecdsa::SigningKey, *};
use eyre::Result;
use log::warn;
use std::error::Error;
use std::str::FromStr;
use std::{convert::TryFrom, sync::Arc};
use tokio::time::timeout;
use tokio::time::Duration;

abigen!(TIOracle, "../contracts/out/TIOracle.sol/TIOracle.json");

pub type OracleStub = TIOracle<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;

pub const CONTRACT_TIMEOUT: u64 = 5000;

//keccak256(abi.encodePacked(coin,price,timestamp))
pub fn get_hash(coin_name: String, price: U256, timestamp: U256) -> [u8; 32] {
    let mut buf = [0 as u8; 32];
    let mut buf2 = [0 as u8; 32];
    price.to_big_endian(&mut buf);
    timestamp.to_big_endian(&mut buf2);
    let packed = [coin_name.as_str().as_bytes(), &buf, &buf2].concat();
    ethers::utils::keccak256(packed.as_slice())
}

// sign price feeding then return signature  and address
pub fn sign_price_info(
    private_key: String,
    coin: String,
    price: u128,
    timestamp: u64,
) -> (String, String) {
    let pk = private_key.parse::<LocalWallet>().unwrap();
    let msg_hash = get_hash(coin, U256::from(price), U256::from(timestamp));
    return (
        pk.sign_hash(H256::from(msg_hash)).to_string(),
        format!("{:?}", pk.address()),
    );
}

// verify signature
pub fn verify_sig(sig: String, coin: String, price: u128, timestamp: u64, address: String) -> bool {
    let sig_obj = Signature::from_str(sig.as_str());
    if let Err(sig_err) = sig_obj {
        warn!("signature verify error: {:?}", sig_err);
        return false;
    }
    let address_result = Address::from_str(address.as_str());
    if let Err(addr_err) = address_result {
        warn!("address error: {:?}", addr_err);
        return false;
    }
    let content_hash = get_hash(coin, U256::from(price), U256::from(timestamp));
    sig_obj
        .unwrap()
        .verify(content_hash, address_result.unwrap())
        .is_ok()
}

pub async fn new(
    private_key: String,
    rpc_url: String,
    contract_address: String,
) -> Result<OracleStub, Box<dyn Error>> {
    let provider = Arc::new({
        // connect to the network
        let provider = Provider::<Http>::try_from(rpc_url.clone())?;
        print!("eth rpc url: {}", rpc_url);
        let chain_id = provider.get_chainid().await?;
        // this wallet's private key
        let wallet = private_key
            .parse::<LocalWallet>()?
            .with_chain_id(chain_id.as_u64());
        SignerMiddleware::new(provider, wallet)
    });
    let hex_addr = contract_address.parse::<Address>()?;
    let oracle_stub = TIOracle::new(hex_addr, provider.clone());
    Ok(oracle_stub)
}

// get_feed_count get the how many times of feeding already committed
pub async fn get_feed_count(
    oracle_stub: &TIOracle<
        ethers::prelude::SignerMiddleware<
            ethers::prelude::Provider<ethers::prelude::Http>,
            ethers::prelude::Wallet<ethers::prelude::k256::ecdsa::SigningKey>,
        >,
    >,
) -> Option<U256> {
    let feed_count: U256;
    let feed_count_result = timeout(
        Duration::from_millis(CONTRACT_TIMEOUT),
        oracle_stub.feed_count().call(),
    )
    .await;
    match feed_count_result {
        Ok(feed_count_obj) => match feed_count_obj {
            Ok(n) => {
                feed_count = n;
            }
            Err(err) => {
                warn!("contract error: {}", err);
                return None;
            }
        },
        Err(timeout_err) => {
            warn!("get feed count err, {}", timeout_err);
            return None;
        }
    }
    Some(feed_count)
}

pub fn from_gwei(gwei: f64) -> U256 {
    u256_from_f64_saturating(gwei * 1.0e9_f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_eth_price_feed() {
        // sign message from your wallet and print out signature produced.
        let node1_pk = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse::<LocalWallet>()
            .unwrap();
        let node2_pk = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
            .parse::<LocalWallet>()
            .unwrap();
        let node3_pk = "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
            .parse::<LocalWallet>()
            .unwrap();
        let h1 = get_hash(
            "eth".into(),
            U256::from(23456 as u32),
            U256::from(1656587035 as u32),
        );
        let h2 = get_hash(
            "eth".into(),
            U256::from(23457 as u32),
            U256::from(1656587035 as u32),
        );
        let h3 = get_hash(
            "eth".into(),
            U256::from(23458 as u32),
            U256::from(1656587035 as u32),
        );
        let sig1 = node1_pk.sign_hash(H256::from(h1));
        let sig2 = node2_pk.sign_hash(H256::from(h2));
        let sig3 = node3_pk.sign_hash(H256::from(h3));
        println!("{}, {}", node1_pk.address(), sig1);
        assert_eq!(
            true,
            sig1.verify(H256::from(h1), node1_pk.address()).is_ok()
        );
        println!("{}, {}", node2_pk.address(), sig2);
        println!("{}, {}", node3_pk.address(), sig3);
    }

    #[test]
    fn generate_btcprice_feed() {
        // sign message from your wallet and print out signature produced.
        let node1_pk = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse::<LocalWallet>()
            .unwrap();
        let node2_pk = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
            .parse::<LocalWallet>()
            .unwrap();
        let node3_pk = "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
            .parse::<LocalWallet>()
            .unwrap();
        let h1 = get_hash(
            "btc".into(),
            U256::from(23456 as u32),
            U256::from(1656587035 as u32),
        );
        let h2 = get_hash(
            "btc".into(),
            U256::from(23457 as u32),
            U256::from(1656587035 as u32),
        );
        let h3 = get_hash(
            "btc".into(),
            U256::from(23458 as u32),
            U256::from(1656587035 as u32),
        );
        println!(
            "{}, {}",
            node1_pk.address(),
            node1_pk.sign_hash(H256::from(h1))
        );
        println!(
            "{}, {}",
            node2_pk.address(),
            node2_pk.sign_hash(H256::from(h2))
        );
        println!(
            "{}, {}",
            node3_pk.address(),
            node3_pk.sign_hash(H256::from(h3))
        );
    }
}
