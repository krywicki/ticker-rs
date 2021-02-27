
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

impl<'a> Widget for Box<dyn StockQuote> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let points: Vec<(f64, f64)> = self.price_points().iter().cloned()
            .enumerate().map(|tuple| (tuple.0 as f64, tuple.1)).collect();

        let dataset = Dataset::default()
            .name(self.symbol())
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(points.as_slice());

        let low = format!("{:.2}", self.low());
        let high = format!("{:.2}",self.high());
        let mid = format!("{:.2}", ((self.high() - self.low()) / 2.0) + self.low());

        let chart = Chart::new(vec![dataset])
            .block(Block::default().title(self.symbol()).borders(Borders::ALL))
            .x_axis(Axis::default()
                //.title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, self.price_points().len() as f64]))
            .y_axis(Axis::default()
                //.title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([self.low(), self.high()])
                .labels([low, mid, high].iter().cloned().map(Span::from).collect()));

        chart.render(area, buf);
    }
}
