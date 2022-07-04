pub mod binance;
pub mod coinbase;
pub mod uniswapv3;
use std::error::Error;
const PRECESIONS: f64 = 1e8;

#[derive(Debug)]
pub struct PairInfo {
    pub symbol: String,
    pub price: u128,
    pub volume: f64,
    pub timestamp: u64,
}

fn convert_bigint_price(s: &String) -> Result<u128, Box<dyn Error>> {
    let price = s.parse::<f64>()?;
    return Ok((price * PRECESIONS) as u128);
}
