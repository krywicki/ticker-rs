#[macro_use]
extern crate serde;

use std::collections::HashMap;
use std::iter::Iterator;

mod error;
pub use error::{Error, ErrorKind};
pub mod tickers;
pub mod de;

pub type Result<T> = std::result::Result<T, Error>;
pub type Symbol = String;

pub struct StockQuote {
    pub symbol: String,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub price: f64,
    pub percent_change: f64,
    pub previous_close: f64,
    pub price_points: Vec<f64>,
}

pub trait TickerAgent {
    fn quotes<T: Iterator> (&self, symbols:T) -> &HashMap<Symbol, Result<&StockQuote>>;
}
