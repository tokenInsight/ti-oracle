use super::expression;
use super::Exchange;
use crate::fetcher::PairInfo;
use async_trait::async_trait;
use eyre::Result;
use reqwest::ClientBuilder;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;
use std::time::Duration;
pub type Piars = Vec<Pair>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    pub prev_close_price: String,
    pub last_price: String,
    pub last_qty: String,
    pub bid_price: String,
    pub bid_qty: String,
    pub ask_price: String,
    pub ask_qty: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub open_time: i64,
    pub close_time: i64,
    pub first_id: i64,
    pub last_id: i64,
    pub count: i64,
}

#[derive(Default, Clone)]
pub struct Binance {}

#[async_trait]
impl Exchange for Binance {
    async fn get_pairs(
        &self,
        symbols: Vec<String>,
    ) -> Result<Vec<PairInfo>, Box<dyn Error + Send + Sync>> {
        let request_url = format!("https://api.binance.com/api/v3/ticker/24hr");
        let timeout = Duration::new(5, 0);
        let client = ClientBuilder::new().timeout(timeout).gzip(true).build()?;
        let response = client.get(&request_url).send().await?;
        let pair_list: Vec<Pair> = response.json().await?;
        let mut crawl_result = Vec::<PairInfo>::new();
        let expand_pairs = expression::expand_symbols(&symbols);
        for pair in &pair_list {
            if symbols.contains(&pair.symbol) || expand_pairs.contains(&pair.symbol) {
                crawl_result.push(PairInfo {
                    symbol: pair.symbol.clone(),
                    price: pair.last_price.parse::<f64>()?,
                    volume: pair.volume.parse::<f64>()?,
                    timestamp: pair.close_time as u64,
                });
            }
        }
        let result = expression::reduce_symbols(&symbols, &crawl_result);
        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fetch() {
        let binance = Binance::default();
        let result = binance
            .get_pairs(vec!["BTCUSDC".into(), "BTCUSDT".into()])
            .await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
