use crate::fetcher::PairInfo;
use std::collections::BTreeMap;

pub struct Symbol {
    pub symbol: String,
}

pub fn expand_symbols(origin_symbols: &Vec<String>) -> Vec<String> {
    let mut symbols = Vec::<String>::new();
    for sym_s in origin_symbols {
        let sym = Symbol {
            symbol: sym_s.clone(),
        };
        if sym.is_expression() {
            for seg in sym.symbol.split(" ") {
                if seg.eq("div") || seg.eq("mul") {
                    continue;
                }
                symbols.push(seg.into());
            }
        } else {
            symbols.push(sym_s.clone());
        }
    }
    return symbols;
}

pub fn reduce_symbols(symbols: &Vec<String>, crawl_result: &Vec<PairInfo>) -> Vec<PairInfo> {
    let mut result = Vec::<PairInfo>::new();
    for symbol in symbols {
        let symb = Symbol {
            symbol: symbol.clone(),
        };
        let pair_ex = symb.eval_pair(crawl_result);
        if !pair_ex.is_none() {
            result.push(pair_ex.unwrap());
        }
    }
    return result;
}

impl Symbol {
    pub fn is_expression(&self) -> bool {
        self.symbol.contains(" ")
    }

    pub fn eval_pair(&self, ref_pairs: &Vec<PairInfo>) -> Option<PairInfo> {
        let price = self.eval_price(ref_pairs);
        if price.is_none() {
            return None;
        }
        let mut base_symbol: String = "".to_string();
        for seg in self.symbol.split(" ") {
            base_symbol = seg.into(); //first one
            break;
        }
        for pair in ref_pairs {
            if pair.symbol.eq(&base_symbol) {
                let mut new_pair = pair.clone();
                new_pair.symbol = self.symbol.clone();
                new_pair.price = price.unwrap();
                return Some(new_pair);
            }
        }
        return None;
    }

    pub fn eval_price(&self, ref_pairs: &Vec<PairInfo>) -> Option<f64> {
        let segments = self.symbol.split(" ");
        let mut value_table = BTreeMap::<String, f64>::new();
        let mut cur_op = "";
        let mut acc: f64 = 1.0;
        for pair in ref_pairs {
            value_table.insert(pair.symbol.clone(), pair.price);
        }
        for seg in segments {
            if seg.len() == 0 || seg.eq(" ") {
                continue;
            }
            if seg == "div" || seg == "mul" {
                cur_op = seg;
                continue;
            }
            if cur_op.eq("") || cur_op.eq("mul") {
                acc = acc
                    * value_table
                        .get(seg)
                        .expect(format!("{} not found", seg).as_str());
            } else if cur_op.eq("div") {
                acc = acc
                    / value_table
                        .get(seg)
                        .expect(format!("{} not found", seg).as_str());
            } else {
                return None;
            }
        }
        return Some(acc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetcher::PairInfo;
    #[test]
    fn test_basic_expression() {
        let x = Symbol {
            symbol: "a div b".into(),
        };
        let y = Symbol {
            symbol: "a mul b".into(),
        };
        let z = Symbol {
            symbol: "a mul b div a".into(),
        };
        let mut ref_pairs = Vec::<PairInfo>::new();
        ref_pairs.push(PairInfo {
            symbol: "a".into(),
            price: 8.0,
            volume: 0.0,
            timestamp: 0,
            exchange: "tiex".into(),
        });
        ref_pairs.push(PairInfo {
            symbol: "b".into(),
            price: 2.0,
            volume: 0.0,
            timestamp: 0,
            exchange: "tiex".into(),
        });
        assert_eq!(x.is_expression(), true);
        let result = x.eval_price(&ref_pairs);
        assert_eq!(result.unwrap(), 4.0);
        let result = y.eval_price(&ref_pairs);
        assert_eq!(result.unwrap(), 16.0);
        println!("{:?}", x.eval_pair(&ref_pairs));
        assert_eq!(z.eval_price(&ref_pairs).unwrap(), 2.0);
    }
}
