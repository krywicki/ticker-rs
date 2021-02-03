use std::iter::Iterator;
use std::collections::HashMap;

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
    //error:json::Value
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
    //high: Vec<json::Value>,
    //low: Vec<json::Value>,
    //close: Vec<json::Value>
}

impl YahooFinanceQuote {
    pub fn symbol(&self) -> &str {
        self.chart.result[0].meta.symbol.as_ref()
    }

    pub fn currency(&self) -> &str {
        self.chart.result[0].meta.currency.as_ref()
    }

    pub fn timezone(&self) -> &str {
        self.chart.result[0].meta.timezone.as_ref()
    }

    pub fn previous_close(&self) -> f64 {
        self.chart.result[0].meta.previous_close
    }

    pub fn exchange_name(&self) -> &str {
        self.chart.result[0].meta.exchange_name.as_ref()
    }
}

pub struct YahooFinanceTicker {
    client: HttpsClient
}

// struct YahooFinanceQuote {
//     symbol:String,
//     // currency:String,
//     // data_granularity:String,
//     // open:Vec<f64>,
//     // high:Vec<f64>,
//     // low:Vec<f64>,
//     // close:Vec<f64>,
//     // volume:Vec<i64>,
// }

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

    pub async fn get_quote<T:AsRef<str>>(&self, symbol: T) -> Result<YahooFinanceQuote> {
        let url = self.url(symbol.as_ref());
        let buf = self.http_get(url).await?;
        let reader = buf.reader();
        let val: YahooFinanceQuote = serde_json::de::from_reader(reader)?;

        Ok(val)
    }

    pub async fn get_quotes<'a, T: Iterator<Item=&'a str>>(&self, symbols:T) -> Result<()> {
        let mut quotes: HashMap::<String, StockQuote> = HashMap::new();

        // get quote bytes for each symbol
        for symbol in symbols {
            let url = self.url(symbol);
            let buf = self.http_get(url).await?;

            let reader = buf.reader();
            let val: YahooFinanceQuote = serde_json::de::from_reader(reader)?;
            print!("{:?}", val);
        }


        Ok(())
    }
}
