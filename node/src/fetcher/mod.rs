pub mod aggregator;
pub mod binance;
pub mod coinbase;
pub mod expression;
pub mod ftx;
pub mod kucoin;
pub mod uniswapv3;

use async_trait::async_trait;
use std::error::Error;
const PRECESIONS_REPRESENT: f64 = 1e8;

#[derive(Debug, Clone)]
pub struct PairInfo {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: u64,
}

#[async_trait]
pub trait Exchange: Send + Sync {
    async fn get_pairs(
        &self,
        symbols: Vec<String>,
    ) -> Result<Vec<PairInfo>, Box<dyn Error + Send + Sync>>;
}
