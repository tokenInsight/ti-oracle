use super::convert_bigint_price;
use super::PairInfo;
use chrono::prelude::*;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;

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

pub async fn get_pairs(symbols: Vec<String>) -> Result<Vec<PairInfo>, Box<dyn Error>> {
    let mut result = Vec::<PairInfo>::new();
    for symbol in symbols {
        let request_url = format!(
            "https://api.pro.coinbase.com/products/{symbol}/ticker",
            symbol = symbol
        );
        //println!("{}", request_url);
        let client = reqwest::Client::new();
        let response = client
            .get(&request_url)
            .header("User-Agent", "ti-oracle")
            .send()
            .await?;
        let pair: Pair = response.json().await?;
        //println!("{}", pair.time.clone());
        let timestamp = DateTime::parse_from_str(&pair.time, "%+")?.timestamp();
        result.push(PairInfo {
            symbol: symbol.clone(),
            price: convert_bigint_price(&pair.price)?,
            volume: pair.volume.parse::<f64>()?,
            timestamp: timestamp as u64,
        });
    }
    return Ok(result);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fetch() {
        let result = get_pairs(vec!["BTC-USDC".into(), "BTC-USD".into()]).await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
