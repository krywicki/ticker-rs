use std::{io, env, sync, thread, time::Duration };
use std::io::Write;

use ticker::TickerAgent;

extern crate ticker;


#[tokio::main]
async fn main() -> Result<(), ticker::QuoteError> {
    let agent = ticker::agent();
    let mut args = env::args();
    args.next();

    while let Some(symbol) = args.next() {
        println!("{}", agent.get_quote(symbol).await?);
    }

    Ok(())
    // let mut app = ui::App::from(quotes);
    // app.run()
}
