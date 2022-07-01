use serde_derive::Deserialize;
use std::vec::Vec;

#[derive(Deserialize)]
pub struct Config {
    pub listen_address: String,
    pub log_level: String,
    pub eth_rpc_url: String,
    pub price_topic: String,
    pub network_topic: String,
    pub contract_address: String,
    pub private_key: String,
    pub coin_name: String,
    pub peers: Vec<String>,
}

impl Config {
    pub fn new(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut c = config::Config::new();
        c.merge(config::File::with_name(file))?;
        Ok(c.try_into()?)
    }
}
