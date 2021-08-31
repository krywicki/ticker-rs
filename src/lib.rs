extern crate serde;

use serde::{ Serialize, Deserialize };
use async_trait::async_trait;
use std::fmt;

mod error;
pub use error::QuoteError;
pub mod agents;
//pub mod ui;

pub type Symbol = String;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct StockQuote {
    symbol: String,
    high: f64,
    low: f64,
    open: f64,
    price: f64,
    percent_change: f64,
    pervious_close: f64,
    price_points: Vec<f64>
}

impl fmt::Display for StockQuote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = serde_json::to_string_pretty(&self)
            .unwrap_or_else(|err| {
                err.to_string()
            });

        write!(f, "{}", val)
    }
}

#[async_trait]
pub trait TickerAgent {
    fn new() -> Self;
    async fn get_quote(&self, symbol:String) -> Result<StockQuote,QuoteError>;
}

/// Get ticker agent
pub fn agent() -> impl TickerAgent {
    agents::YahooFinanceAgent::new()
}

pub trait FloatMinMax {
    fn f64_min(&mut self) -> f64;
    fn f64_max(&mut self) -> f64;
}

impl<T> FloatMinMax for T where T: Iterator<Item=f64> {
    fn f64_max(&mut self) -> f64 {
        self.fold(f64::NAN, |a,b| a.max(b) )
    }

    fn f64_min(&mut self) -> f64 {
        self.fold(f64::NAN, |a,b| a.min(b) )
    }
}
