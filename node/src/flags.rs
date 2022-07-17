use serde_derive::Deserialize;
use std::collections::BTreeMap;
use std::vec::Vec;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub listen_address: String,
    pub web_address: String,
    pub log_level: String,
    pub eth_rpc_url: String,
    pub contract_address: String,
    pub private_key: String,
    pub coin_name: String,
    pub peers: Vec<String>,
    pub mappings: BTreeMap<String, Vec<String>>,
    pub feed_interval: u64,
    pub fee_per_gas: f64,
}

impl Config {
    pub fn new(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut c = config::Config::new();
        c.merge(config::File::with_name(file))?;
        Ok(c.try_into()?)
    }
}
