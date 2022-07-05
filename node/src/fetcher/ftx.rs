use crate::fetcher::PairInfo;
use crate::processor::utils;
use async_trait::async_trait;
use eyre::Result;
use reqwest::ClientBuilder;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;
use std::time::Duration;

use super::Exchange;
use super::PRECESIONS;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairList {
    pub success: bool,
    pub result: Vec<Pair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub name: String,
    pub enabled: bool,
    pub post_only: bool,
    pub price_increment: f64,
    pub size_increment: f64,
    pub min_provide_size: f64,
    pub last: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub price: Option<f64>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub future_type: Option<String>,
    pub base_currency: Option<String>,
    pub is_etf_market: bool,
    pub quote_currency: Option<String>,
    pub underlying: Option<String>,
    pub restricted: bool,
    pub high_leverage_fee_exempt: bool,
    pub large_order_threshold: f64,
    pub change1h: f64,
    pub change24h: f64,
    pub change_bod: f64,
    pub quote_volume24h: f64,
    pub volume_usd24h: f64,
    pub price_high24h: f64,
    pub price_low24h: f64,
    pub tokenized_equity: Option<bool>,
}
#[derive(Default)]
pub struct Ftx {}

#[async_trait]
impl Exchange for Ftx {
    async fn get_pairs(&self, symbols: Vec<String>) -> Result<Vec<PairInfo>, Box<dyn Error>> {
        let request_url = format!("https://ftx.com/api/markets");
        let timeout = Duration::new(5, 0);
        let client = ClientBuilder::new().timeout(timeout).gzip(true).build()?;
        let response = client.get(&request_url).send().await?;
        let pair_list: PairList = response.json().await?;
        let mut result = Vec::<PairInfo>::new();
        for pair in &pair_list.result {
            if symbols.contains(&pair.name) {
                let price = pair.price.expect("price invalid");
                result.push(PairInfo {
                    symbol: pair.name.clone(),
                    price: (price * PRECESIONS) as u128,
                    volume: (pair.quote_volume24h / price),
                    timestamp: utils::timestamp() as u64,
                });
            }
        }
        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fetch() {
        let ftx = Ftx::default();
        let result = ftx
            .get_pairs(vec!["BTC/USD".into(), "BTC/USDT".into()])
            .await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
