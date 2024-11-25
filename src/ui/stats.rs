use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Bar, BarChart, BarGroup, Block, Borders},
    Frame,
};

pub fn render_stats(f: &mut Frame, area: &Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightGreen))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(format!("Stats {}", 5))
        .title_alignment(Alignment::Center)
        .bg(Color::Rgb(30, 34, 42))
        .title_style(Style::default().fg(Color::Yellow).bold());
    f.render_widget(header_block, *area);

    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let lol = BarChart::default()
        .bar_width(1)
        .bar_gap(1)
        .group_gap(3)
        .value_style(Style::new().hidden())
        .bar_style(Style::new().yellow().on_red())
        .label_style(Style::new().white())
        .data(&[("B0", 1), ("B1", 12), ("B2", 0), ("B3", 0)])
        .max(1000);
    f.render_widget(lol, inner_area);
}
