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

    crossterm::terminal::enable_raw_mode().expect("enabling raw mode");
    io::stdout().execute(crossterm::terminal::EnterAlternateScreen).expect("enter alternate screen");

    let backend = tui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = tui::Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::terminal::EnterAlternateScreen).unwrap();
    terminal.clear()?;
    terminal.draw(|f| f.render_widget(app, f.size()))?;

    let (tx, rx) = sync::mpsc::channel();
    thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(200)).expect("event polling") {
                if let event::Event::Key(key) = event::read().expect("can read events") {
                    tx.send(ui::Event::Input(key)).expect("tx - event");
                }
            }
        }
    });


    //== loop rx events
    loop {
        match rx.recv()? {

            ui::Event::Input(e) => match e.code {
                // quit app
                event::KeyCode::Char('q') => {
                    crossterm::terminal::disable_raw_mode().expect("disable raw mode");
                    terminal.show_cursor()?;
                    break;
                }
                _ => {}
            }
            ui::Event::Tick => {}
        }
    }

    crossterm::terminal::disable_raw_mode().expect("enabling raw mode");
    io::stdout().execute(crossterm::terminal::LeaveAlternateScreen).expect("exit alternate screen");
    Ok(())
}
