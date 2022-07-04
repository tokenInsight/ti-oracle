pub mod binance;
pub mod coinbase;
pub mod uniswapv3;

const PRECESIONS: f64 = 1e8;

#[derive(Debug)]
pub struct PairInfo {
    pub symbol: String,
    pub price: u128,
    pub volume: f64,
    pub timestamp: u64,
}
