use std::io;
use termion::raw::IntoRawMode;
use tui::{
    Terminal,
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap, GraphType
    },
    Frame,
};

extern crate ticker;
use ticker::StockTicker;



#[tokio::main]
async fn main() -> Result<(), ticker::Error> {
    let ticker = StockTicker::new();
    let quote = ticker.quote("PLUG").await.unwrap();


    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;


    let points: Vec<(f64, f64)> = quote.price_points().iter().cloned()
        .enumerate().map(|tuple| (tuple.0 as f64, tuple.1)).collect();

    let dataset = Dataset::default()
        .name("price points")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(points.as_slice());

    let low = format!("{:.2}", quote.low());
    let high = format!("{:.2}",quote.high());
    let mid = format!("{:.2}", ((quote.high() - quote.low()) / 2.0));

    let chart = Chart::new(vec![dataset])
        .block(Block::default().title("Price Points"))
        .x_axis(Axis::default()
            .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([0.0, quote.price_points().len() as f64]))
            //.labels(["0", "5.0", ""].iter().cloned().map(Span::from).collect()))
        .y_axis(Axis::default()
            .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([quote.low(), quote.high()])
            .labels([low, mid, high].iter().cloned().map(Span::from).collect()));

    terminal.draw(|f| {
        f.render_widget(chart, f.size());
    })?;

    Ok(())
}
