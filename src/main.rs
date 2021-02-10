extern crate ticker_rs;
use ticker_rs::StockTicker;
use ticker_rs::TickerAgent;
use ticker_rs::tickers::YahooFinanceTicker;

#[tokio::main]
async fn main() {
    // let agent = YahooFinanceTicker::new();
    // let ticker = StockTicker::from(agent);
    let ticker = StockTicker::new();

    let quote = ticker.quote("PLUG").await.unwrap();



    //let vals = ticker.get_quotes(["PLUG", "NFLX"].iter()).await;
    println!("{:#?}", quote.price());
}
