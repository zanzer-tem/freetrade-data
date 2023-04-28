mod freetrade_data_loader {
    use serde::{Deserialize, Serialize};
    use std::env;

    const FREETRADE_DATA_ENDPOINT:&str = "https://sheets.googleapis.com/v4/spreadsheets/14Ep-CmoqWxrMU8HshxthRcdRW8IsXvh3n2-ZHVCzqzQ/values/Freetrade%20Universe!A:I?key=";
    // Deserializer for bool

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ApiSymbolData {
        pub title: String,
        pub long_title: String,
        pub subtitle: String,
        pub currency: String,
        pub isa_eligible: String,
        #[serde(skip_serializing)]
        _sipp_eligible: String,
        #[serde(skip_serializing)]
        _isin: String,
        pub mic: String,
        pub symbol: String,

    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ApiResponse {
        #[serde(skip_serializing)]
        _range: String,
        #[serde(skip_serializing)]
        _major_dimension: String,
        values: Vec<ApiSymbolData>,
    }

    pub fn load_data() -> Vec<ApiSymbolData> {
        let google_api_key = env::var("GOOGLE_API_KEY").expect("Error: GOOGLE_API_KEY not found");
        let mut data = Vec::new();
        let client = reqwest::blocking::Client::new();
        let resp = match client
            .get(String::from(FREETRADE_DATA_ENDPOINT) + &google_api_key)
            .send()
        {
            Ok(resp) => {
                let api_response = resp.json::<ApiResponse>().unwrap();
                data = api_response.values;
            }
            Err(err) => panic!("Error: {}", err),
        };
        data
    }
}

pub mod freetrade_data {
    use std::collections::HashMap;
    use std::fmt;

    
    #[derive(Debug, Copy, Clone)]    
    pub enum Exchange{
        NYSE,
        NASDAQ,
        LSE,
        UNKNOWN
    }

    impl fmt::Display for Exchange {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}\n", self);
            Ok(())
        }
    }

    pub struct Symbol {
        pub symbol: String,
        pub name: String,
        pub long_name: String
    }

    

    pub struct SymbolData {
        pub symbol: Symbol,
        pub sector: String,
        pub exchange: Exchange,
        pub isa_eligible: bool,
    }

    impl fmt::Display for SymbolData {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Ticker: {} {}\nExchange: {:?}\n", self.symbol.symbol, self.symbol.name, self.exchange);
            Ok(())
        }
    }

    pub struct FreetradeData {
        data: HashMap<String, SymbolData>,
    }

    impl FreetradeData {
        fn load_data() -> HashMap<String, SymbolData> {
            let mut data = HashMap::new();
            let api_data = crate::freetrade_data_loader::load_data();

            for row in &api_data {
                data.insert(
                    row.symbol.clone(),
                    SymbolData {
                        symbol: Symbol {
                            symbol: row.symbol.clone(),
                            name: row.title.clone(),
                            long_name: row.long_title.clone()
                        },
                        isa_eligible: match row.isa_eligible == "TRUE" {
                            true => true,
                            false => false,
                        },
                        sector: row.subtitle.clone(),
                        exchange: match row.mic.as_str() {
                            "XNAS" => Exchange::NASDAQ,
                            "XNYS" => Exchange::NYSE,
                            "XLON" => Exchange::LSE,
                            _ => Exchange::UNKNOWN
                        }
                    },
                );
            }
            data
        }

        pub fn symbol(&self, symbol: &str) -> Option<&SymbolData> {
            self.data.get(symbol)
        }

        pub fn symbols(&self) -> Vec<&SymbolData> {
            self.data
            .iter()
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        pub fn symbols_in_exchange(&self, exchange: Exchange) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|(_, value)| { 
                matches!(&value.exchange, exchange)
            })
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        pub fn isa_eligible_symbols(&self) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|(_, value)| { 
                value.isa_eligible
            })
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        pub fn is_isa_eligible(&self, symbol: &str) -> bool {
            let symbol_uppercase = symbol.to_uppercase();
            self.data.contains_key(symbol_uppercase.as_str())
                &&  self
                        .data
                        .get(symbol_uppercase.as_str())
                        .unwrap()
                        .isa_eligible
        }

        pub fn new() -> FreetradeData {
            FreetradeData {
                data: Self::load_data(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freetrade_data::FreetradeData;

    #[test]
    fn present_and_eligible_works() {
        let freetrade_data = FreetradeData::new();
        assert_eq!(freetrade_data.is_isa_eligible("AAPL"), true);
    }

    #[test]
    fn present_and_not_eligible_works() {
        let freetrade_data = FreetradeData::new();
        assert_eq!(freetrade_data.is_isa_eligible("WALB"), false);
    }

    #[test]
    fn not_present_and_eligible_works() {
        let freetrade_data = FreetradeData::new();
        assert_eq!(freetrade_data.is_isa_eligible("ZZZZZZ"), false);
    }
}
