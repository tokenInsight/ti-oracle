use crate::processor::utils;

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
pub struct Pair {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub pool: Pool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub id: String,
    #[serde(rename = "token0Price")]
    pub token0price: String,
    #[serde(rename = "token1Price")]
    pub token1price: String,
    pub volume_token0: String,
    pub volume_token1: String,
    pub token0: Token0,
    pub token1: Token1,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token0 {
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token1 {
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct QueryRequest {
    pub query: String,
}

#[derive(Default, Clone)]
pub struct UniswapV3 {}

#[async_trait]
impl Exchange for UniswapV3 {
    async fn get_pairs(&self, symbols: Vec<String>) -> Result<Vec<PairInfo>, Box<dyn Error>> {
        let mut result = Vec::<PairInfo>::new();
        for symbol in symbols {
            let request_url = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3";
            //println!("{}", request_url);
            let timeout = Duration::new(5, 0);
            let client = ClientBuilder::new().timeout(timeout).build()?;
            let fmt_str = r#"
        {
            pool(id: "{pool_id}") {
              id
              token0Price
              token1Price
              volumeToken0
              volumeToken1
              token0 { symbol }
              token1 { symbol }
            }
          }
        "#;
            let query_params = fmt_str.to_string().replace("{pool_id}", &symbol);
            let query: QueryRequest = QueryRequest {
                query: query_params,
            };
            let content = serde_json::to_string(&query).unwrap();
            //println!("{}", content);
            let response = client.post(request_url).body(content).send().await?;
            //println!("{}", response.text().await?);
            let pair: Pair = response.json().await?;
            let adjust_weight = 0.0063; //TODO: later, we should fetch 24hours volume
            result.push(PairInfo {
                symbol: symbol.clone(),
                price: convert_bigint_price(&pair.data.pool.token1price)?,
                volume: pair.data.pool.volume_token0.parse::<f64>()? * adjust_weight,
                timestamp: utils::timestamp() as u64, //TODO timestamp
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
        let uni = UniswapV3::default();
        let result = uni
            .get_pairs(vec![
                "0x99ac8ca7087fa4a2a1fb6357269965a2014abc35".into(),
                "0x9db9e0e53058c89e5b94e29621a205198648425b".into(),
            ])
            .await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
