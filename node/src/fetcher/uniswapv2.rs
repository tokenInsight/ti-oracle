use super::expression;
use super::Exchange;
use super::PairInfo;
use crate::processor::utils;
use async_trait::async_trait;
use reqwest::ClientBuilder;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::error::Error;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub pairs: Vec<Pair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
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

#[derive(Default, Clone)]
pub struct UniswapV2 {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct QueryRequest {
    pub query: String,
}

#[async_trait]
impl Exchange for UniswapV2 {
    async fn get_pairs(
        &self,
        symbols: Vec<String>,
    ) -> Result<Vec<PairInfo>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::<PairInfo>::new();
        for symbol in expression::expand_symbols(&symbols) {
            let request_url = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2";
            //println!("{}", request_url);
            let timeout = Duration::new(5, 0);
            let client = ClientBuilder::new().timeout(timeout).build()?;
            let fmt_str = r#"
            query {
                pairs(where:{id_in:["{pair_id}"]})  {
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
            let query_params = fmt_str.to_string().replace("{pair_id}", &symbol);
            let query: QueryRequest = QueryRequest {
                query: query_params,
            };
            let content = serde_json::to_string(&query).unwrap();
            //println!("{}", content);
            let response = client.post(request_url).body(content).send().await?;
            //println!("{}", response.text().await?);
            let response_json: Response = response.json().await?;
            let adjust_weight = 0.0063; //TODO: later, we should fetch 24hours volume
            if response_json.data.pairs.len() > 0 {
                result.push(PairInfo {
                    symbol: symbol.clone(),
                    price: response_json.data.pairs[0].token1price.parse::<f64>()?,
                    volume: response_json.data.pairs[0].volume_token0.parse::<f64>()?
                        * adjust_weight,
                    timestamp: utils::timestamp() as u64, //TODO timestamp
                });
            }
        }
        let result = expression::reduce_symbols(&symbols, &result);
        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fetch() {
        let uni = UniswapV2::default();
        let result = uni
            .get_pairs(vec![
                "0x004375dff511095cc5a197a54140a24efef3a416".into(),
                "0xbb2b8038a1640196fbe3e38816f3e67cba72d940 div 0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc".into(),
            ])
            .await;
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
    }
}
