use ethers::prelude::{k256::ecdsa::SigningKey, *};
use eyre::Result;
use std::error::Error;
use std::{convert::TryFrom, sync::Arc};

abigen!(TIOracle, "../contracts/out/TIOracle.sol/TIOracle.json");

type OracleStub = TIOracle<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;

//keccak256(abi.encodePacked(coin,price,timestamp))
pub fn get_hash(coin_name: String, price: U256, timestamp: U256) -> [u8; 32] {
    let mut buf = [0 as u8; 32];
    let mut buf2 = [0 as u8; 32];
    price.to_big_endian(&mut buf);
    timestamp.to_big_endian(&mut buf2);
    let packed = [coin_name.as_str().as_bytes(), &buf, &buf2].concat();
    ethers::utils::keccak256(packed.as_slice())
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
