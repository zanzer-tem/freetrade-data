pub mod freetrade_data {

    use std::collections::HashMap;
    use std::env;
    use serde::{de::{self, IntoDeserializer},Deserialize};
    use std::fmt;

    
    const FREETRADE_DATA_ENDPOINT:&str = "https://sheets.googleapis.com/v4/spreadsheets/14Ep-CmoqWxrMU8HshxthRcdRW8IsXvh3n2-ZHVCzqzQ/values/Freetrade%20Universe!A:K?key=";

    #[derive(Debug, PartialEq)]
    #[repr(u8)]
    pub enum Market {
        USD,
        GBP,
        SEK,
        EUR,
        UNKNOWN,
    }
    
    impl Market {
        
        pub fn as_symbol(&self) -> String {
            match self {
                Self::USD => String::from("$"),
                Self::GBP => String::from("£"),
                Self::EUR => String::from("€"),
                Self::SEK => String::from("kr"),
                _ => String::from("$")
            }
        }

        pub fn from_symbol(symbol: &str) -> Result<Market, ()> {
            let exchange_symbol = symbol.chars().nth(0);
            
            match exchange_symbol {
                None => Ok(Market::UNKNOWN),
                Some(c) => match c {
                                    '$'  => Ok(Market::USD),
                                    '£'  => Ok(Market::GBP),
                                    '€' => Ok(Market::EUR),
                                    _      => Ok(Market::UNKNOWN),
                                }
            }
            
        }

        pub fn from_name(name: &str) -> Result<Market, ()> {
            match name {
                "NASDAQ"  => Ok(Market::USD),
                "NYSE" => Ok(Market::USD),
                "NYSE ARCA" => Ok(Market::USD),
                "LSE"  => Ok(Market::GBP),
                _      => Err(()),
            }
        }

       pub fn from_exchange(exchange: &Mic) -> Result<Market, ()> {
            match exchange {
                Mic::XNAS  => Ok(Market::USD),
                Mic::XNYS => Ok(Market::USD),
                Mic::XLON  => Ok(Market::GBP),
                _      => Err(()),
            }
        }
    }

    #[derive(Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Currency {
        #[serde(rename = "eur")]
        EUR,
        #[serde(rename = "gbp")]
        GBP,
        #[serde(rename = "usd")]
        USD,
        #[serde(rename = "sek")]
        SEK,
        UNKNOWN
    }


    #[derive(Deserialize, Debug,PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum Mic {
        XETR,
        XLON,
        XNAS,
        XNYS,
        XLIS,
        PINK,
        XHEL,
        XWBO,
        XAMS,
        XBRU,
        XSTO,
        UNKNOWN
    }

    impl fmt::Display for Mic {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}\n", self);
            Ok(())
        }
    }

    #[derive(Deserialize, Debug,Eq, Ord, PartialEq, PartialOrd)]
    #[serde(rename_all = "camelCase")]
    pub struct SymbolData {
        pub title: String,
        pub long_title: String,
        pub subtitle: String,
        #[serde(deserialize_with="deserialize_currency")]
        pub currency: Currency,
        #[serde(deserialize_with="deserialize_bool")]
        pub isa_eligible: bool,
        #[serde(deserialize_with="deserialize_bool")]
        pub sipp_eligible: bool,
        pub isin: String,
        #[serde(deserialize_with="deserialize_mic")]
        pub mic: Mic,
        pub symbol: String,
        pub fractional_enabled: String,
        #[serde(deserialize_with="deserialize_bool")]
        pub plus_only: bool
    }

    impl SymbolData {
        pub fn market(&self) -> Result<Market,()> {
            Market::from_exchange(&self.mic)
        }
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct FreetradeApiResponse {
        #[serde(skip_serializing)]
        _range: String,
        #[serde(skip_serializing)]
        _major_dimension: String,
        values: Vec<SymbolData>,
    }

    fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s: &str = de::Deserialize::deserialize(deserializer)?;

        match s {
            "TRUE" => Ok(true),
            _ => Ok(false)
        }
    }

    fn deserialize_currency<'de, D>(deserializer: D) -> Result<Currency, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s: &str = de::Deserialize::deserialize(deserializer)?;

        match s {
            "Currency" => Ok(Currency::UNKNOWN),
            _ => Currency::deserialize(s.into_deserializer())
        }
    }

    fn deserialize_mic<'de, D>(deserializer: D) -> Result<Mic, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s: &str = de::Deserialize::deserialize(deserializer)?;

        match s {
            "MIC" => Ok(Mic::UNKNOWN),
            _ => Mic::deserialize(s.into_deserializer())
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FreetradeDataKey {
        symbol: String,
        mic: Mic
    }

    fn load_data() -> HashMap<FreetradeDataKey, SymbolData>{
        let google_api_key = env::var("GOOGLE_API_KEY").expect("Error: GOOGLE_API_KEY not found");
        let mut data = HashMap::new();
        let client = reqwest::blocking::Client::new();
        let resp = match client
            .get(String::from(FREETRADE_DATA_ENDPOINT) + &google_api_key)
            .send()
        {
            Ok(resp) => {
                let mut api_response = resp.json::<FreetradeApiResponse>().unwrap();
                // remove the column headers
                api_response.values.swap_remove(0);
                data = api_response.values.into_iter().map(|row| (FreetradeDataKey{
                    symbol: row.symbol.clone(),
                    mic: row.mic
                }, row)).collect();
            }
            Err(err) => panic!("Error: {}", err),
        };
        data
    }

    pub struct FreetradeData {
        pub data: HashMap<FreetradeDataKey, SymbolData>,
    }

    impl FreetradeData {
        pub fn new() -> FreetradeData {
            FreetradeData {
                data: load_data(),
            }
        }

        ///
        /// Returns all isa eligible symbols with the given ticker
        /// 
        pub fn symbol(&self, symbol: &str) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|s| s.1.symbol.as_str().eq(symbol))
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        ///
        /// Returns all isa eligible symbols with the given ticker
        /// 
        pub fn isa_eligible_symbol(&self, symbol: &str) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|s| s.1.symbol.as_str().eq(symbol) && s.1.isa_eligible)
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        ///
        /// Returns a symbol in a given exchange if it exists
        /// 
        pub fn symbol_in_exchange(&self, symbol: &str, mic: Mic) -> Option<&SymbolData> {
            self.data.get(&FreetradeDataKey { symbol: String::from(symbol), mic: mic })
        }

        //
        // Returns all symbols across all markets & exchanges
        //
        pub fn symbols(&self) -> Vec<&SymbolData> {
            self.data
            .iter()
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        pub fn symbols_in_exchange(&self, exchange: &Mic) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|(_, value)| { 
                matches!(&value.mic, exchange)
            })
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        //
        // Returns all symbols in the specified market
        //
        pub fn symbols_in_market(&self, market: &Market) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|(_, value)| { 
                matches!(&value.market(), market)
            })
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        //
        // Returns all symbols in the specified markets
        //
        pub fn symbols_in_markets(&self, markets: Vec<&Market>) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|(_, value)| { 
                match value.market() {
                    Ok(Market) => markets.contains(&&value.market().unwrap()),
                    Err(_) => false
                }
            })
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        ///
        /// Returns all isa eligible symbols across all markets and exchanges
        /// 
        pub fn isa_eligible_symbols(&self) -> Vec<&SymbolData> {
            self.data
            .iter()
            .filter(|(_, value)| { 
                value.isa_eligible
            })
            .map(|(_, symbol_data)| symbol_data)
            .collect()
        }

        
        pub fn is_isa_eligible(&self, symbol: &str, mic:Mic) -> bool {
            let symbol_uppercase = symbol.to_uppercase();
            let freetradeDataKey = FreetradeDataKey{
                symbol: symbol_uppercase,
                mic: mic
            };
            self.data.contains_key(&freetradeDataKey)
                &&  self
                        .data
                        .get(&freetradeDataKey)
                        .unwrap()
                        .isa_eligible
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
        assert_eq!(freetrade_data.is_isa_eligible("AAPL", freetrade_data::Mic::XNAS), true);
    }

    #[test]
    fn present_and_not_eligible_works() {
        let freetrade_data = FreetradeData::new();
        assert_eq!(freetrade_data.is_isa_eligible("WALB", freetrade_data::Mic::XNYS), false);
    }

    #[test]
    fn not_present_and_eligible_works() {
        let freetrade_data = FreetradeData::new();
        assert_eq!(freetrade_data.is_isa_eligible("ZZZZZZ", freetrade_data::Mic::XLON), false);
    }
}
