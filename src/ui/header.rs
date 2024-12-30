use std::{path::Path, str::FromStr};

use devicons::FileIcon;
use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const WATCHED_DIR_LABEL: &str = " Watched Directory:";

pub fn render_header(f: &mut Frame, area: &Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightGreen))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(format!("Await-Modify-Do v{}", APP_VERSION))
        .title_style(Style::default().fg(Color::Yellow).bold())
        // .bg(Color::Rgb(30, 34, 42))
        .title_alignment(Alignment::Center);
    f.render_widget(header_block, *area);

    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let lines = vec![render_watched_dir(), render_mode(), render_command()];
    let text = Text::from(lines);
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    f.render_widget(p, inner_area);
}

fn render_args() -> Result<Vec<Span<'static>>, ()> {
    let mut spans = vec![];
    match std::env::args().skip(1).next() {
        Some(p) => match which::which(&p) {
            Ok(_) => spans.push(Span::styled(p, Style::default().fg(Color::Green))),
            Err(_) => {
                let path = Path::new(&p);
                if path.exists() {
                    let icon = FileIcon::from(path);
                    spans.push(Span::styled(
                        icon.to_string(),
                        Style::default()
                            .fg(Color::from_str(icon.color).unwrap_or(Color::LightBlue)),
                    ));
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        p,
                        Style::default().fg(Color::LightCyan).underlined(),
                    ));
                } else {
                    spans.push(Span::styled(
                        p,
                        Style::default().fg(Color::Red).bold().underlined(),
                    ))
                }
            }
        },
        None => {
            return Err(());
        }
    }

    for arg in std::env::args().skip(2) {
        let path = Path::new(&arg);
        if arg.starts_with('-') {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(arg, Style::default().fg(Color::LightRed)));
        } else if path.is_dir() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled("", Style::default().fg(Color::LightBlue)));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(arg, Style::default().fg(Color::LightCyan)).underlined());
        } else if path.is_file() {
            spans.push(Span::raw(" "));
            let icon = FileIcon::from(path);
            let color = Color::from_str(icon.color).unwrap_or(Color::LightBlue);
            spans.push(Span::styled(icon.to_string(), Style::default().fg(color)));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(arg, Style::default().fg(Color::LightCyan)).underlined());
        } else {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(arg, Style::default().fg(Color::White)));
        }
    }

    Ok(spans)
}

fn render_command() -> Line<'static> {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("Command:", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" "),
        Span::styled("$", Style::default().fg(Color::Green).bold()),
        Span::raw(" "),
    ];
    match render_args() {
        Ok(x) => {
            spans.extend(x);
        }
        Err(_) => {
            spans.push(Span::styled("No command", Style::default().fg(Color::Red)));
        }
    }
    let line = Line::from(spans);
    line
}

fn render_mode() -> Line<'static> {
    Line::from(vec![
        Span::raw(" "),
        Span::styled("Mode:", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" "),
        Span::styled("Normal", Style::default().fg(Color::LightGreen)),
    ])
}

fn render_watched_dir() -> Line<'static> {
    match std::env::current_dir() {
        Ok(p) => {
            Line::from(vec![
                Span::raw(" "),
                Span::styled(WATCHED_DIR_LABEL, Style::default().fg(Color::Yellow).bold()),
                Span::raw(" "),
                Span::styled(
                    // Directory icon
                    " ",
                    Style::default().fg(Color::LightBlue),
                ),
                Span::styled(
                    p.to_string_lossy().to_string(),
                    Style::default().fg(Color::LightCyan).underlined(),
                ),
            ])
        }

        Err(e) => Line::from(vec![
            Span::raw(" "),
            Span::styled(WATCHED_DIR_LABEL, Style::default().fg(Color::Yellow).bold()),
            Span::raw(" "),
            Span::styled(
                format!("Error getting current directory {}", e),
                Style::default().fg(Color::Red),
            ),
        ]),
    }
}
