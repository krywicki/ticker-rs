
use std::io;
use termion::raw::IntoRawMode;
use tui::{
    Terminal,
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    buffer::Buffer,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap, GraphType, Widget
    },
    Frame,
};


use crate::{
    Result, StockQuote
};


pub fn draw(quote: &Box<dyn StockQuote>) -> Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend);

    Ok(())
}

fn draw_quote_table(quote: &Box<dyn StockQuote>, area: Rect, buf: &mut Buffer) {
    let rows = vec![
        Row::new(vec!["Symbol", quote.symbol()])
            .style(Style::default().fg(Color::Green)),

        Row::new(vec!["Price".into(), format!("{:>20}",quote.price())])
            .style(Style::default().fg(Color::Green))
    ];

    let table = Table::new(rows)
        .header(Row::new(vec!["", ""]))
        .block(Block::default()
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
        )
        .widths(&[Constraint::Percentage(25), Constraint::Percentage(75)]);

    table.render(area, buf);
}

fn draw_quote_chart(quote: &Box<dyn StockQuote>, area: Rect, buf: &mut Buffer) {
    let points: Vec<(f64, f64)> = quote.price_points().iter().cloned()
            .enumerate().map(|tuple| (tuple.0 as f64, tuple.1)).collect();

    let dataset = Dataset::default()
        .name(quote.symbol())
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(points.as_slice());

    let low = format!("{:.2}", quote.low());
    let high = format!("{:.2}",quote.high());
    let mid = format!("{:.2}", ((quote.high() - quote.low()) / 2.0) + quote.low());

    let chart = Chart::new(vec![dataset])
        .block(Block::default().borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
        .x_axis(Axis::default()
            //.title(Span::styled("X Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([0.0, quote.price_points().len() as f64]))
        .y_axis(Axis::default()
            //.title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([quote.low(), quote.high()])
            .labels([low, mid, high].iter().cloned().map(Span::from).collect()));

    chart.render(area, buf);
}

impl<'a> Widget for Box<dyn StockQuote> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .constraints([Constraint::Length(35), Constraint::Length(area.width - 35)].as_ref())
            .direction(Direction::Horizontal)
            .split(area);

        draw_quote_table(&self, chunks[0], buf);
        draw_quote_chart(&self, chunks[1], buf);
    }
}
