

struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub show_chart: bool,
    pub progress: f64,
    pub sparkline: Signal<>
}
