use std::iter::Iterator;
use std::collections::HashMap;

use async_trait::async_trait;

use hyper::{ Client, http::uri::Uri};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::{Body, StatusCode};
use hyper::body::Buf;

use serde::{ Deserialize, Deserializer };
use serde_json::Value;
use serde_json::value as json;

use crate::de::FromJsonResponse;
use crate::{TickerAgent, Result, StockQuote};
use crate::error::{Error, ErrorKind};
use crate::FloatMinMax;

type Connector = HttpsConnector<HttpConnector>;
type HttpsClient = Client<Connector, Body>;


///
/// Deserializer helpers
///
pub mod de {
    use super::*;
    use serde_json::Value;
    use serde::de::Error as Error;

    fn unwrap_num<'de, D>(num: &Value) -> std::result::Result<f64, D::Error>
        where D: Deserializer<'de>
    {
        if let Value::Number(num) = num {
            if let Some(num) = num.as_f64() {
                return Ok(num);
            }
        }
        Err(Error::custom("expected Some num"))
    }

    pub fn yahoo_nums<'de, D>(d:D) -> std::result::Result<Vec<f64>, D::Error>
        where D: Deserializer<'de>
    {
        let vals = Vec::<serde_json::Value>::deserialize(d);
        let vals:Vec::<&serde_json::Value> = vals.iter().flatten().collect();

        let mut _vals:Vec<f64> = vec!();
        for val in vals {
            match val {
                Value::Number(_) => {
                    _vals.push(unwrap_num::<D>(val)?);
                },

                Value::Array(array) => {
                    for _val in array {
                        _vals.push(unwrap_num::<D>(_val)?);
                    }
                },

                _ => {
                    return Err(Error::custom("unexpected type"))
                }
            }
        }

        Ok(_vals)
    }
}

#[derive(Debug, Deserialize)]
pub struct YahooFinanceQuote { chart:Chart }

#[derive(Debug, Deserialize)]
pub struct Chart {
    result:Vec<ChartResult>,
    error:json::Value
}

#[derive(Debug, Deserialize)]
pub struct ChartResult {
    meta:Meta,
    indicators: Indicators
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    currency: String,
    symbol: String,
    timezone: String,

    #[serde(rename="regularMarketPrice")]
    regular_market_price: f64,

    #[serde(rename="previousClose")]
    previous_close: f64,

    #[serde(rename="exchangeName")]
    exchange_name: String
}

#[derive(Debug, Deserialize)]
pub struct Indicators{ quote: Vec<Quote> }

#[derive(Debug, Deserialize)]
pub struct Quote {
    #[serde(deserialize_with="de::yahoo_nums")]
    open: Vec<f64>,

    #[serde(deserialize_with="de::yahoo_nums")]
    high: Vec<f64>,

    #[serde(deserialize_with="de::yahoo_nums")]
    low: Vec<f64>,

    #[serde(deserialize_with="de::yahoo_nums")]
    close: Vec<f64>
}

impl YahooFinanceQuote {
    fn meta(&self) -> &Meta {
        &self.chart.result[0].meta
    }

    fn quote(&self) -> &Quote {
        &self.chart.result[0].indicators.quote[0]
    }
}

impl StockQuote for YahooFinanceQuote {
    fn symbol(&self) -> &str {
        self.meta().symbol.as_ref()
    }

    fn high(&self) -> f64 {
        self.quote().high.iter().cloned().f64_max()
    }

    fn low(&self) -> f64 {
        self.quote().low.iter().cloned().f64_min()
    }

    fn open(&self) -> f64 {
        *self.quote().open.last().unwrap_or(&0f64)
    }

    fn price(&self) -> f64 {
        self.meta().regular_market_price
    }

    fn previous_close(&self) -> f64 {
        self.meta().previous_close
    }

    fn percent_change(&self) -> f64 {
        -((self.previous_close() - self.price()) / self.previous_close()) * 100.0f64
    }

    fn price_points(&self) -> &Vec<f64> {
        &self.quote().open
    }
}

pub struct YahooFinanceTicker {
    client: HttpsClient
}

impl YahooFinanceTicker {
    pub fn new() -> YahooFinanceTicker {
        YahooFinanceTicker {
            client: Client::builder().build(Connector::new())
        }
    }

    fn url(&self, symbol:&str) -> Uri {
        format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}\
            ?region=USincludePrePost=false&interval=2m&range=1d&corsDomain=finance.yahoo.com&.tsrc=finance",
            symbol
        ).parse().unwrap()
    }

    async fn http_get(&self, url:Uri) -> Result<impl Buf> {
        // http get
        let resp = self.client.get(url).await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::new(
                ErrorKind::HttpError,
                resp.status().canonical_reason().unwrap_or("unknown"))
            );
        }

        // return bytes
        let body = resp.into_body();
        let buf = hyper::body::aggregate(body).await?;
        Ok(buf)
    }

    pub async fn get_quote_json<T:AsRef<str>>(&self, symbol:T) -> Result<serde_json::Value> {
        let url = self.url(symbol.as_ref());
        let buf = self.http_get(url).await?;

        let value:serde_json::Value = serde_json::de::from_reader(buf.reader())?;
        Ok(value)
    }

    // pub async fn get_quote<T:AsRef<str>>(&self, symbol: T) -> Result<YahooFinanceQuote> {
    //     let url = self.url(symbol.as_ref());
    //     let buf = self.http_get(url).await?;
    //     let reader = buf.reader();
    //     let val: YahooFinanceQuote = serde_json::de::from_reader(reader)?;

    //     if Value::Null != val.chart.error {
    //         Err(Error::new(ErrorKind::Unknown, val.chart.error.to_string()))
    //     } else {
    //         Ok(val)
    //     }
    // }

    // pub async fn get_quotes<'a,S, T>(&self, symbols:T) -> HashMap<String, Result<YahooFinanceQuote>>
    //     where S: AsRef<str>,
    //     T: Iterator<Item=S>
    // {
    //     let mut quotes: HashMap::<String,  Result<YahooFinanceQuote>> = HashMap::new();

    //     // get quote bytes for each symbol
    //     for symbol in symbols {
    //         quotes.insert(symbol.as_ref().into(), self.get_quote(symbol).await);
    //     }

    //     quotes
    // }
}

#[async_trait]
impl TickerAgent for YahooFinanceTicker {
    // async fn get_quotes<'e,I>(&self, symbols:I) -> HashMap<String, Result<Box<dyn StockQuote>>>
    //     where I: Iterator<Item=&'e dyn AsRef<str>>
    // {
    //     let mut quotes: HashMap<String, Result<Box<dyn StockQuote>>> = HashMap::new();
    //     quotes
    // }

    async fn get_quote(&self, symbol: String) -> Result<Box<dyn StockQuote>> {
        let url = self.url(symbol.as_ref());
        let buf = self.http_get(url).await?;
        let reader = buf.reader();
        let val: YahooFinanceQuote = serde_json::de::from_reader(reader)?;

        if Value::Null != val.chart.error {
            Err(Error::new(ErrorKind::Unknown, val.chart.error.to_string()))
        } else {
            Ok(Box::new(val))
        }
    }
}
