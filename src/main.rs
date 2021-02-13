

extern crate ticker_rs;
use ticker_rs::StockTicker;



#[tokio::main]
async fn main() {
    let ticker = StockTicker::new();
    let quote = ticker.quote("PLUG").await.unwrap();



    //let vals = ticker.get_quotes(["PLUG", "NFLX"].iter()).await;
    println!("{:#?}", quote.price());
}
