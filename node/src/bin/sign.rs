// use the eyre crate for easy idiomatic error handling
use eyre::Result;
use std::env;
// use the ethers_signers crate to manage LocalWallet and Signer
use ethers::prelude::*;

// this file is a test case data generation program

fn get_hash(coin_name: String, price: U256, timestamp: U256) -> [u8; 32] {
    let mut buf = [0 as u8; 32];
    let mut buf2 = [0 as u8; 32];
    price.to_big_endian(&mut buf);
    timestamp.to_big_endian(&mut buf2);
    let packed = [coin_name.as_str().as_bytes(), &buf, &buf2].concat();
    ethers::utils::keccak256(packed.as_slice())
}

fn generateEthPriceFeed() -> Result<()> {
    // sign message from your wallet and print out signature produced.
    let node1_pk = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse::<LocalWallet>()?;
    let node2_pk = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
        .parse::<LocalWallet>()?;
    let node3_pk = "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
        .parse::<LocalWallet>()?;
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
    Ok(())
}

fn generateBTCPriceFeed() -> Result<()> {
    // sign message from your wallet and print out signature produced.
    let node1_pk = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse::<LocalWallet>()?;
    let node2_pk = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
        .parse::<LocalWallet>()?;
    let node3_pk = "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
        .parse::<LocalWallet>()?;
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
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // sign message from your wallet and print out signature produced.
    generateEthPriceFeed()?;
    println!("++++++++");
    generateBTCPriceFeed()?;
    Ok(())
}
