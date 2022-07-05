use super::convert_bigint_price;
use super::Exchange;
use super::PairInfo;
use async_trait::async_trait;
use reqwest::ClientBuilder;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub code: String,
    pub data: Pair,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub time: i64,
    pub symbol: String,
    pub buy: String,
    pub sell: String,
    pub change_rate: String,
    pub change_price: String,
    pub high: String,
    pub low: String,
    pub vol: String,
    pub vol_value: String,
    pub last: String,
    pub average_price: String,
    pub taker_fee_rate: String,
    pub maker_fee_rate: String,
    pub taker_coefficient: String,
    pub maker_coefficient: String,
}

#[derive(Default, Clone)]
pub struct Kucoin {}

#[async_trait]
impl Exchange for Kucoin {
    async fn get_pairs(&self, symbols: Vec<String>) -> Result<Vec<PairInfo>, Box<dyn Error>> {
        let mut result = Vec::<PairInfo>::new();
        for symbol in symbols {
            let request_url = format!(
                "https://api.kucoin.com/api/v1/market/stats?symbol={symbol}",
                symbol = symbol
            );
            let timeout = Duration::new(5, 0);
            let client = ClientBuilder::new().timeout(timeout).build()?;
            let response = client
                .get(&request_url)
                .header("User-Agent", "ti-oracle")
                .send()
                .await?;
            let rsps: Response = response.json().await?;
            //println!("{}", pair.time.clone());
            let pair = rsps.data;
            result.push(PairInfo {
                symbol: symbol.clone(),
                price: convert_bigint_price(&pair.buy)?,
                volume: pair.vol.parse::<f64>()?,
                timestamp: pair.time as u64,
            });
        }
        return Ok(result);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fetch() {
        let kucoin = Kucoin::default();
        let result = kucoin
            .get_pairs(vec!["BTC-USDC".into(), "BTC-USDT".into()])
            .await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
