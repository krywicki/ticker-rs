use std::iter::Iterator;

use async_trait::async_trait;

use reqwest;
use serde::{ Deserialize, Deserializer };
use serde_json::value as json;

use crate::error::QuoteError;
use crate::{TickerAgent, StockQuote};
use crate::FloatMinMax;
///
/// Deserializer helpers
///
pub mod de {
    use super::*;
    use serde_json::Value;
    use serde::de::Error as Error;

    #[derive(Debug, Deserialize)]
    pub struct Chart {
        pub result:Vec<ChartResult>,
        pub error:json::Value
    }

    #[derive(Debug, Deserialize)]
    pub struct ChartResult {
        pub meta:Meta,
        pub indicators: Indicators
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all="camelCase")]
    pub struct Meta {
        pub currency: String,
        pub symbol: String,
        pub timezone: String,
        pub regular_market_price: f64,
        pub previous_close: f64,
        pub exchange_name: String
    }

    #[derive(Debug, Deserialize)]
    pub struct Indicators{ pub quote: Vec<Quote> }

    #[derive(Debug, Deserialize)]
    pub struct Quote {
        #[serde(deserialize_with="yahoo_nums")]
        pub open: Vec<f64>,

        #[serde(deserialize_with="yahoo_nums")]
        pub high: Vec<f64>,

        #[serde(deserialize_with="yahoo_nums")]
        pub low: Vec<f64>,

        #[serde(deserialize_with="yahoo_nums")]
        pub close: Vec<f64>
    }

    /// unwrap json number
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

    /// deserialize yahoo numbers
    pub fn yahoo_nums<'de, D>(d:D) -> std::result::Result<Vec<f64>, D::Error>
        where D: Deserializer<'de>
    {
        let mut nums:Vec<f64> = vec!();
        let vals = Vec::<serde_json::Value>::deserialize(d);
        let vals:Vec::<&serde_json::Value> = vals.iter().flatten().collect();

        for val in vals {
            match val {
                Value::Number(_) => nums.push(unwrap_num::<D>(val)?),
                Value::Null => continue, // skip nulls,
                Value::Array(array) => {
                    array.iter()
                        .try_for_each(|num| -> Result<(), D::Error> {
                            let n = unwrap_num::<D>(num)?;
                            nums.push(n);
                            Ok(())
                        })?;
                },
                _ => return Err(Error::custom("unexpected type"))
            }
        }

        Ok(nums)
    }
}

#[derive(Debug, Deserialize)]
pub struct YahooFinanceQuote {
    chart: json::Value,
    error: Option<json::Value>
}

impl de::Chart {
    fn meta(&self) -> &de::Meta {
        &self.result[0].meta
    }

    fn quote(&self) -> &de::Quote {
        &self.result[0].indicators.quote[0]
    }

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
        *self.quote().open.first().unwrap_or(&0f64)
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

impl From<de::Chart> for StockQuote {
    fn from(yahoo: de::Chart) -> StockQuote {
        StockQuote {
            symbol: yahoo.symbol().into(),
            high: yahoo.high(),
            low: yahoo.low(),
            open: yahoo.open(),
            price: yahoo.price(),
            pervious_close: yahoo.previous_close(),
            percent_change: yahoo.percent_change(),
            price_points: yahoo.price_points().to_vec()
        }
    }
}

pub struct YahooFinanceAgent {
    client: reqwest::Client,
}

impl YahooFinanceAgent {

    fn map_http_err(&self, symbol:&String, message:String, err: reqwest::Error) -> QuoteError {
        QuoteError {
            symbol: symbol.to_string(),
            message: message,
            detail: Some(err.to_string()),
            source: Some(Box::new(err))
        }
    }

    async fn http_get_quote(&self, symbol: &String) -> Result<reqwest::Response, QuoteError> {
        //== http get from yahoo finance
        let var = self.client
            .get(format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol))
            .form(&[
                ("region",          "US"),
                ("lang",            "en-US"),
                ("includePrePost",  "true"),
                ("interval",        "2m"),
                ("range",           "1d"),
                ("corsDomain",      "finance.yahoo.com"),
                (".tsrc",           "finance")
            ])

            .send()
            .await
            .map_err(|err| self.map_http_err(&symbol, "Request Failure".into(), err))?

            .error_for_status()
            .map_err(|err| self.map_http_err(&symbol, "Request Failure".into(), err))?;

        Ok(var)
    }
}

#[async_trait]
impl TickerAgent for YahooFinanceAgent {
    fn new() -> YahooFinanceAgent {
        YahooFinanceAgent {
            client: reqwest::Client::new()
        }
    }

    async fn get_quote(&self, symbol: String) -> Result<StockQuote, QuoteError> {

        // http get yahoo quote
        let response = self.http_get_quote(&symbol).await?;

        //== deserialize
        let quote = response
            .json::<YahooFinanceQuote>()
            .await
            .map_err(|err| self.map_http_err(&symbol, "Parse Failure".into(), err))?;

        //== check errors
        if let Some(err) = quote.error {
            return Err(QuoteError{
                symbol: symbol,
                message: "Missing Data".into(),
                detail: Some(err.to_string()),
                source: None
            });
        }

        //== convert into stock quote
        let q = serde_json::from_value::<de::Chart>(quote.chart)
            .map_err(|err| {
                QuoteError {
                    symbol: symbol.to_string(),
                    message: "Parse Failure".into(),
                    detail: Some(err.to_string()),
                    source: Some(Box::new(err))
                }
            })?
            .into();

        Ok(q)
    }
}
