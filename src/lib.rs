#[macro_use]
extern crate serde;

use async_trait::async_trait;

mod error;
pub use error::{Error, ErrorKind};
pub mod tickers;
pub mod de;

pub type Result<T> = std::result::Result<T, Error>;
pub type Symbol = String;

pub trait StockQuote {
    fn symbol(&self) -> &str;
    fn high(&self) -> f64;
    fn low(&self) -> f64;
    fn open(&self) -> f64;
    fn price(&self) -> f64;
    fn percent_change(&self) -> f64;
    fn previous_close(&self) -> f64;
    fn price_points(&self) -> &Vec<f64>;
}

#[async_trait]
pub trait TickerAgent {
    async fn get_quote(&self, symbol:String) -> Result<Box<dyn StockQuote>>;
}

pub struct StockTicker<T> {
    agent: T
}

impl<T:TickerAgent> StockTicker<T> {

    pub fn from(agent:T) -> Self {
        StockTicker {
            agent
        }
    }

    pub async fn quote<S:AsRef<str>>(&self, symbol:S) -> Result<Box<dyn StockQuote>> {
        return self.agent.get_quote(symbol.as_ref().into()).await;
    }
}

impl StockTicker<tickers::YahooFinanceTicker> {
    pub fn new() -> Self {
        StockTicker {
            agent: tickers::YahooFinanceTicker::new()
        }
    }
}

pub trait FloatMinMax {
    fn f64_min(&mut self) -> f64;
    fn f64_max(&mut self) -> f64;
}

impl<T> FloatMinMax for T where T: Iterator<Item=f64> {
    fn f64_max(&mut self) -> f64 {
        self.fold(f64::NAN, f64::max)
    }

    fn f64_min(&mut self) -> f64 {
        self.fold(f64::NAN, f64::min)
    }
}


pub fn get_ticker() -> Box<dyn TickerAgent> {
    Box::new(tickers::YahooFinanceTicker::new())
}
