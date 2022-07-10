use super::ftx::Ftx;
use super::kucoin::Kucoin;
use super::{binance, coinbase, uniswapv2, uniswapv3, Exchange, PairInfo, PRECESIONS_REPRESENT};
use binance::Binance;
use coinbase::Coinbase;
use futures::future;
use log::{debug, info, warn};
use std::collections::BTreeMap;
use std::error::Error;
use std::vec::Vec;
use strum_macros::Display;
use uniswapv2::UniswapV2;
use uniswapv3::UniswapV3;

pub struct Aggregator {
    data_sources: BTreeMap<String, Box<dyn Exchange>>,
    mappings: BTreeMap<String, Vec<String>>,
}

pub fn new(mappings: BTreeMap<String, Vec<String>>) -> Aggregator {
    let mut agg = Aggregator {
        data_sources: BTreeMap::new(),
        mappings: mappings,
    };
    agg.data_sources
        .insert("binance".into(), Box::new(Binance::default()));
    agg.data_sources
        .insert("coinbase".into(), Box::new(Coinbase::default()));
    agg.data_sources
        .insert("uniswapv3".into(), Box::new(UniswapV3::default()));
    agg.data_sources
        .insert("uniswapv2".into(), Box::new(UniswapV2::default()));
    agg.data_sources
        .insert("ftx".into(), Box::new(Ftx::default()));
    agg.data_sources
        .insert("kucoin".into(), Box::new(Kucoin::default()));
    agg
}

#[derive(Debug, Display)]
pub enum AggError {
    NoEnoughVolumes(f64),
    NoPairs(String),
}
impl std::error::Error for AggError {}

impl Aggregator {
    pub async fn get_price(&self) -> Result<u128, Box<dyn Error + Send + Sync>> {
        let mut exchagne_names = Vec::<String>::new();
        let tasks = self
            .data_sources
            .iter()
            .filter(|item| self.mappings.contains_key(item.0))
            .map(|(ex_name, exchange)| {
                let symbols = self.mappings[ex_name].clone();
                let result = exchange.get_pairs(symbols);
                exchagne_names.push(ex_name.clone());
                return result;
            });
        let all_exchanges: Vec<Result<Vec<PairInfo>, Box<dyn Error + Send + Sync>>> =
            future::join_all(tasks).await;
        let mut total_volume = 0 as f64;
        let mut all_pairs = Vec::<&PairInfo>::new();
        let mut offset: usize = 0;
        for exchange in &all_exchanges {
            match exchange {
                Ok(pairs) => {
                    info!("*** {} ***", exchagne_names[offset]);
                    for pair in pairs {
                        total_volume += pair.volume;
                        all_pairs.push(pair);
                        info!(
                            " +--- {} -> {} vol:{}",
                            pair.symbol, pair.price, pair.volume
                        );
                    }
                }
                Err(err) => {
                    warn!("{}", err)
                }
            }
            offset += 1;
        }
        if total_volume < 1.0 {
            return Err(Box::new(AggError::NoEnoughVolumes(total_volume)));
        }
        calc_weighted_price(all_pairs)
    }
}

fn remove_outliers(mut all_pairs: Vec<&PairInfo>) -> Vec<&PairInfo> {
    let mut result = Vec::<&PairInfo>::new();
    all_pairs.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    let n = all_pairs.len();
    let p25 = (n as f64 * 0.25) as usize;
    let p75 = (n as f64 * 0.75) as usize;
    let p25_price = all_pairs[p25].price;
    let p75_price = all_pairs[p75].price;
    let iqr = p75_price - p25_price;
    let mut lower_bound = 0.0;
    if p25_price > (iqr * 1.5) {
        lower_bound = p25_price - (iqr * 1.5);
    }
    let upper_bound = p75_price + (iqr * 1.5);
    for pair_price in all_pairs {
        if pair_price.price < lower_bound || pair_price.price > upper_bound {
            warn!("outlier skipped: {:?}", pair_price);
            continue;
        }
        result.push(pair_price);
    }
    result
}

fn calc_weighted_price(
    all_pairs_original: Vec<&PairInfo>,
) -> Result<u128, Box<dyn Error + Send + Sync>> {
    debug!("pairs count:{}", all_pairs_original.len());
    let all_pairs = remove_outliers(all_pairs_original);
    debug!("oufter remove outliers, pairs count:{}", all_pairs.len());
    let mut avg_price = 0.0 as f64;
    let total_volume = all_pairs
        .iter()
        .map(|a| a.volume)
        .reduce(|a, b| a + b)
        .unwrap();
    for pair in all_pairs {
        debug!(" ++ {}", pair.price);
        avg_price += pair.price as f64 * pair.volume / total_volume;
    }
    debug!("avg : {}", avg_price);
    Ok((avg_price * PRECESIONS_REPRESENT) as u128)
}

#[cfg(test)]
mod tests {
    use crate::fetcher::aggregator;
    use crate::flags;
    #[tokio::test]
    async fn test_agg() {
        let cfg = flags::Config::new("./config/node.yaml").unwrap();
        let agg = aggregator::new(cfg.mappings);
        let weighted_price = agg.get_price().await.unwrap();
        println!("weighted price:{}", weighted_price);
    }
}
