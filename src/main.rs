extern crate ticker_rs;
use ticker_rs::tickers::YahooFinanceTicker;

#[tokio::main]
async fn main() {
    let ticker = YahooFinanceTicker::new();
    //ticker.get_quote_json("PLUG").await.unwrap();
    let val = ticker.get_quote("PLUG").await.unwrap();
    println!("{:#?}", val)
}
