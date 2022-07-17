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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub code: String,
    pub msg: String,
    pub data: Vec<Pair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub inst_type: String,
    pub inst_id: String,
    pub last: String,
    pub last_sz: String,
    pub ask_px: String,
    pub ask_sz: String,
    pub bid_px: String,
    pub bid_sz: String,
    pub open24h: String,
    pub high24h: String,
    pub low24h: String,
    pub vol_ccy24h: String,
    pub vol24h: String,
    pub ts: String,
    pub sod_utc0: String,
    pub sod_utc8: String,
}

#[derive(Default, Clone)]
pub struct OkEx {}

pub const NAME: &str = "okex";

#[async_trait]
impl Exchange for OkEx {
    async fn get_pairs(
        &self,
        symbols: Vec<String>,
    ) -> Result<Vec<PairInfo>, Box<dyn Error + Send + Sync>> {
        let request_url = format!("https://www.okex.com/api/v5/market/tickers?instType=SPOT");
        let timeout = Duration::new(5, 0);
        let client = ClientBuilder::new().timeout(timeout).gzip(true).build()?;
        let response = client.get(&request_url).send().await?;
        let pair_list: Response = response.json().await?;
        let mut crawl_result = Vec::<PairInfo>::new();
        let expand_pairs = expression::expand_symbols(&symbols);
        for pair in &pair_list.data {
            if symbols.contains(&pair.inst_id) || expand_pairs.contains(&pair.inst_id) {
                crawl_result.push(PairInfo {
                    symbol: pair.inst_id.clone(),
                    price: pair.last.parse::<f64>()?,
                    volume: pair.vol24h.parse::<f64>()?,
                    timestamp: pair.ts.parse::<u64>()?,
                    exchange: NAME.into(),
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
        let okex = OkEx::default();
        let result = okex
            .get_pairs(vec!["BTC-USDC".into(), "BTC-USDT".into()])
            .await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
