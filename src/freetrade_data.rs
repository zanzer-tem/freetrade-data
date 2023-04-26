mod freetrade_data_loader {
    use serde::{Deserialize, Serialize};
    use std::env;

    const FREETRADE_DATA_ENDPOINT:&str = "https://sheets.googleapis.com/v4/spreadsheets/14Ep-CmoqWxrMU8HshxthRcdRW8IsXvh3n2-ZHVCzqzQ/values/Freetrade%20Universe!E:I?key=";
    // Deserializer for bool

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ApiSymbolData {
        pub isa_eligible: String,
        #[serde(skip_serializing)]
        _sipp_eligible: String,
        #[serde(skip_serializing)]
        _isin: String,
        #[serde(skip_serializing)]
        _mic: String,
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

    struct SymbolData {
        symbol: String,
        isa_eligible: bool,
    }

    impl fmt::Display for SymbolData {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{} {}", self.symbol, self.isa_eligible);
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
                        symbol: row.symbol.clone(),
                        isa_eligible: match row.isa_eligible == "TRUE" {
                            true => true,
                            false => false,
                        },
                    },
                );
            }
            data
        }

        pub fn symbols(&self) -> Vec<String> {
            self.data
                .iter()
                .map(|(_, value)| String::from(&value.symbol))
                .collect()
        }

        pub fn contains(&self, symbol: &str, isa_eligible: bool) -> bool {
            let symbol_uppercase = symbol.to_uppercase();
            self.data.contains_key(symbol_uppercase.as_str())
                && (isa_eligible
                    && self
                        .data
                        .get(symbol_uppercase.as_str())
                        .unwrap()
                        .isa_eligible)
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
        assert_eq!(freetrade_data.contains("AAPL", true), true);
    }

    #[test]
    fn present_and_not_eligible_works() {
        let freetrade_data = FreetradeData::new();
        assert_eq!(freetrade_data.contains("WALB", true), false);
    }

    #[test]
    fn not_present_and_eligible_works() {
        let freetrade_data = FreetradeData::new();
        assert_eq!(freetrade_data.contains("ZZZZZZ", true), false);
    }
}
