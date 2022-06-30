use ethers::prelude::{k256::ecdsa::SigningKey, *};
use eyre::Result;
use std::error::Error;
use std::{convert::TryFrom, sync::Arc};

abigen!(TIOracle, "../contract/out/TIOracle.sol/TIOracle.json");

type OracleStub = TIOracle<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;

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
