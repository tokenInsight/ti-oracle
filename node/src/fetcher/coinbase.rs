use super::expression;
use super::Exchange;
use super::PairInfo;
use async_trait::async_trait;
use chrono::prelude::*;
use reqwest::ClientBuilder;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub ask: String,
    pub bid: String,
    pub volume: String,
    #[serde(rename = "trade_id")]
    pub trade_id: i64,
    pub price: String,
    pub size: String,
    pub time: String,
}

#[derive(Default, Clone)]
pub struct Coinbase {}

#[async_trait]
impl Exchange for Coinbase {
    async fn get_pairs(
        &self,
        symbols: Vec<String>,
    ) -> Result<Vec<PairInfo>, Box<dyn Error + Send + Sync>> {
        let mut crawl_result = Vec::<PairInfo>::new();
        for symbol in expression::expand_symbols(&symbols) {
            let request_url = format!(
                "https://api.pro.coinbase.com/products/{symbol}/ticker",
                symbol = symbol
            );
            //println!("{}", request_url);
            let timeout = Duration::new(5, 0);
            let client = ClientBuilder::new().timeout(timeout).build()?;
            let response = client
                .get(&request_url)
                .header("User-Agent", "ti-oracle")
                .send()
                .await?;
            let pair: Pair = response.json().await?;
            //println!("{}", pair.time.clone());
            let timestamp = DateTime::parse_from_str(&pair.time, "%+")?.timestamp();
            crawl_result.push(PairInfo {
                symbol: symbol.clone(),
                price: pair.price.parse::<f64>()?,
                volume: pair.volume.parse::<f64>()?,
                timestamp: timestamp as u64,
            });
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
        let coinbase = Coinbase::default();
        let result = coinbase
            .get_pairs(vec!["BTC-USD".into(), "BTC-USDT".into()])
            .await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
