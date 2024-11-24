use ratatui::{
    layout::{Alignment, Constraint, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, LineGauge, Row, Sparkline, Table},
    Frame,
};

pub fn render_output(f: &mut Frame, area: &Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightGreen))
        .border_type(ratatui::widgets::BorderType::Double)
        .title(format!("output {}", 5))
        .title_alignment(Alignment::Center)
        .title_style(Style::default().fg(Color::Yellow).bold())
        .bg(Color::Rgb(30, 34, 42));
    f.render_widget(header_block, *area);

    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // let LineGuage = LineGauge::default()
    //     .filled_style(Style::default().fg(Color::Green))
    //     .ratio(0.5);
    // f.render_widget(LineGuage, inner_area);
    // let gauge = Gauge::default()
    //     .block(Block::default().title("Gauge").borders(Borders::ALL))
    //     .gauge_style(Style::default().fg(Color::Green))
    //     .ratio(0.5);
    // f.render_widget(gauge, inner_area);
    // let sparkline = Sparkline::default()
    //     .block(Block::default().title("Sparkline").borders(Borders::ALL))
    //     .style(Style::default().fg(Color::Green))
    //     .data(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    // f.render_widget(sparkline, inner_area);
}
