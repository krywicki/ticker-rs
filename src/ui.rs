
use std::{rc::Rc, io, thread, time::Duration, sync};
use tui::{
    buffer::Buffer,
    layout::{ Constraint, Direction, Layout, Rect, Alignment },
    style::{
        Color, Modifier, Style
    },
    symbols,
    text::{ Span, Text },
    widgets::{ Axis, Block, Borders, Cell, Chart, Dataset, GraphType,
        List, ListItem, ListState, Paragraph, Row, StatefulWidget, Table, Widget,
        TableState
    }
};
use crossterm::{
    event, execute, ExecutableCommand
};

use crate::{Error, StockQuote};


type BoxQuote=Box<dyn StockQuote>;
type QuoteList=Rc<Vec<BoxQuote>>;

pub struct App {
    quotes: QuoteList,
    symbols: SymbolsWidget,
    chart: ChartWidget
}

impl App {
    pub fn from<I>(quotes: I) -> Self
        where I: IntoIterator<Item=BoxQuote>
    {
        let quotes: QuoteList = Rc::new(quotes.into_iter().collect());
        App {
            quotes: quotes.clone(),
            symbols: SymbolsWidget::new(quotes.clone()),
            chart: ChartWidget::new()
        }
    }

    pub fn run(self) -> Result<(), Error> {
        crossterm::terminal::enable_raw_mode().expect("enabling raw mode");
        io::stdout().execute(crossterm::terminal::EnterAlternateScreen).expect("enter alternate screen");

        let backend = tui::backend::CrosstermBackend::new(io::stdout());
        let mut terminal = tui::Terminal::new(backend)?;

        execute!(terminal.backend_mut(), crossterm::terminal::EnterAlternateScreen).unwrap();
        terminal.clear()?;
        terminal.draw(|f| f.render_widget(self, f.size()))?;

        let (tx, rx) = sync::mpsc::channel();
        thread::spawn(move || {
            loop {
                if event::poll(Duration::from_millis(200)).expect("event polling") {
                    if let event::Event::Key(key) = event::read().expect("can read events") {
                        tx.send(Event::Input(key)).expect("tx - event");
                    }
                }
            }
        });


        //== loop rx events
        loop {
            match rx.recv()? {

                Event::Input(e) => match e.code {
                    // quit app
                    event::KeyCode::Char('q') => {
                        crossterm::terminal::disable_raw_mode().expect("disable raw mode");
                        terminal.show_cursor()?;
                        break;
                    }
                    _ => {}
                }
                Event::Tick => {}
            }
        }

        crossterm::terminal::disable_raw_mode().expect("enabling raw mode");
        io::stdout().execute(crossterm::terminal::LeaveAlternateScreen).expect("exit alternate screen");
        Ok(())
    }
}

struct SymbolsWidget {
    quotes: QuoteList,
    state: TableState
}

impl SymbolsWidget {
    fn new(quotes: QuoteList) -> Self {
        SymbolsWidget {
            quotes,
            state: TableState::default()
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.quotes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            },
            None => 0
        };
        self.state.select(Some(i));
    }
}

impl<'a> From<&'a BoxQuote> for Row<'a> {
    fn from(quote: &'a BoxQuote) -> Self {
        let prefix;
        let color;
        let modifier;

        //== Determine colors and prefix +/-
        if quote.percent_change() > 0.0 {
            prefix = "+";
            color = Color::LightGreen;
        } else {
            prefix = "";
            color = Color::Red;
        }

        //== Determine if % change should blink
        if quote.percent_change().abs() >= 5.0 {
            modifier = Modifier::RAPID_BLINK;
        } else {
            modifier = Modifier::empty();
        }

        //== Create and return row
        Row::new(vec![
            Cell::from(
                Text::styled(
                    quote.symbol(),
                    Style::default().fg(Color::Yellow)
                )
            ),
            Cell::from(
                Text::styled(
                    format!("{}{:.2}%", prefix, quote.percent_change()),
                    Style::default().fg(color)
                )
            )
        ])
    }
}

impl Widget for SymbolsWidget {

    fn render(mut self, area:Rect, buf: &mut Buffer) {
        let color = Color::Rgb(78, 78, 78);
        let rows: Vec<Row> = self.quotes.iter().map(|q| Row::from(q)).collect();
        let table = Table::new(rows)
            .block(Block::default().borders(Borders::all()))
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .highlight_style(Style::default().bg(color))
            .highlight_symbol(" >> ");

        self.state.select(Some(0));

        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}

struct QuoteWidget<'a> {
    quote: &'a BoxQuote
}

impl<'a> QuoteWidget<'a> {
    fn new(quote: &'a BoxQuote) -> Self {
        QuoteWidget {
            quote
        }
    }
}

impl<'a> Widget for QuoteWidget<'a> {
    fn render(self, area:Rect, buf: &mut Buffer) {
        let title = Span::styled(
            format!(" {} ", self.quote.symbol()),
            Style::default().fg(Color::Yellow)
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        block.render(area, buf);
    }
}


struct ChartWidget {}

impl ChartWidget {
    fn new() -> Self {
        ChartWidget {}
    }
}

impl<'a> Widget for App {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(30), Constraint::Percentage(75)])
            .split(area);

        self.symbols.render(chunks[0], buf);
        //SymbolsWidget::new(self.quotes.clone()).render(chunks[0], buf, &mut self.state);
        QuoteWidget::new(&self.quotes[0]).render(chunks[1], buf);
    }
}

pub enum Event<I> {
    Input(I),
    Tick
}

pub const HEIGHT:u16 = 15;

fn draw_quote_info_header(quote: &Box<dyn StockQuote>, area: Rect, buf: &mut Buffer) {

    //== create stock symbol widget
    let span = Span::styled(quote.symbol(), Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD));
    let symbol = Paragraph::new(span).alignment(Alignment::Left);

    //== create percent change widget
    let color = if quote.percent_change() < 0.0 { Color::Red } else { Color::Green };
    let prefix = if quote.percent_change() < 0.0 { "" } else { "+" };
    let mut style = Style::default().fg(color);

    if  quote.percent_change().abs()  > 5.0 {
        style = style.add_modifier(Modifier::RAPID_BLINK);
    }

    let span = Span::styled(format!("{}{:.2} %", prefix, quote.percent_change()), style);
    let perc_change = Paragraph::new(span).alignment(Alignment::Right);

    //== create block widget for header underline
    let block = Block::default().borders(Borders::BOTTOM);
    let inner_area = block.inner(area);

    //== divide header into left and right parts
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .direction(Direction::Horizontal)
        .split(inner_area);

    //== render widgets
    block.render(area, buf);
    symbol.render(chunks[0], buf);
    perc_change.render(chunks[1], buf);
}

fn draw_quote_info_body(quote: &Box<dyn StockQuote>, area: Rect, buf: &mut Buffer) {

    // list of field/value tuples
    let values = vec![
        ("Price", format!("${:.2}", quote.price())),
        ("Previous Close", format!("${:.2}", quote.previous_close())),
        ("Open", format!("${:.2}", quote.open())),
        ("High", format!("${:.2}", quote.high())),
        ("Low", format!("${:.2}", quote.low())),
    ];

    // create row chunks
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage((100/values.len()) as u16); values.len()]) // 5 rows
        .split(area);

    // render field/value tuples in table like manner
    //   *note: done this way b/c widget::Table does not have alignment or row spacing available.
    for i in 0..values.len() {
        let ref tuple = values[i];
        let field = Paragraph::new(tuple.0);
        let value = Paragraph::new(tuple.1.as_ref()).alignment(Alignment::Right);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50);2])
            .split(rows[i]);

        field.render(cols[0], buf);
        value.render(cols[1], buf);
    }
}

fn draw_quote_info(quote: &Box<dyn StockQuote>, area: Rect, buf: &mut Buffer) {

    //== render right side, vertical separator between quote info and chart
    let block = Block::default()
        .borders(Borders::RIGHT);

    let inner_area =  block.inner(area);
    block.render(area, buf);

    //== split inner block area into header and body for quote info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(inner_area.height - 2)])
        .horizontal_margin(1)
        .split(inner_area);


    draw_quote_info_header(&quote, chunks[0], buf);
    draw_quote_info_body(&quote, chunks[1], buf);
}

fn draw_quote_chart(quote: &Box<dyn StockQuote>, area: Rect, buf: &mut Buffer) {
    //== create dataset for rendering stock price points in line chart

    // get price points as [...,(x,y),...] coords for line chart
    let points: Vec<(f64, f64)> = quote.price_points().iter().cloned()
        .enumerate().map(|tuple| (tuple.0 as f64, tuple.1)).collect();

    // create dataset
    let color = if quote.price() >= quote.previous_close() { Color::Green } else { Color::Red };
    let dataset = Dataset::default()
        //.name(quote.symbol())
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(points.as_slice());


    //== create x-axis line to represent previous close
    let prev_close_points: Vec<(f64, f64)> = quote.price_points().iter()
        .enumerate().map(|tuple| (tuple.0 as f64, quote.previous_close())).collect();

    // create previous close dataset
    let prev_close_dataset = Dataset::default()
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::DarkGray))
        .data(prev_close_points.as_slice());


    // create line chart to render datasets
    let chart = Chart::new(vec![prev_close_dataset, dataset, ])
        .x_axis(Axis::default()
            .style(Style::default().fg(Color::White))
            .bounds([0.0, quote.price_points().len() as f64])
        )
        .y_axis(Axis::default()
            .style(Style::default().fg(Color::White))
            .bounds([quote.low(), quote.high()])
        );

    chart.render(area, buf);
}

impl<'a> Widget for Box<dyn StockQuote> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        //let rect = Rect::new(area.x, area.y, area.width, HEIGHT);
        let rect = area;
        let block = Block::default().borders(Borders::ALL);
        let inner_rect = block.inner(rect);

        block.render(rect, buf);

        let chunks = Layout::default().direction(Direction::Horizontal)
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(35), Constraint::Percentage(75)])
            .split(inner_rect);

        draw_quote_info(&self, chunks[0], buf);
        draw_quote_chart(&self, chunks[1], buf);
    }
}
