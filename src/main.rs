use std::{io, env, sync, thread, time::Duration };
use std::io::Write;
use crossterm::{
    event, execute, ExecutableCommand
};

use tui;

extern crate ticker;
use ticker::ui;
use ticker::StockTicker;


#[tokio::main]
async fn main() -> Result<(), ticker::Error> {
    let ticker = StockTicker::new();
    let mut quotes = vec![];
    let mut args = env::args();

    args.next();
    while let Some(symbol) = args.next() {
        quotes.push(ticker.quote(symbol).await.unwrap());
    }

    let app = ui::App::from(quotes);
    app.run()
}
