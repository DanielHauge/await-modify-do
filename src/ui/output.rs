use std::{process::exit, time::Instant};

use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::process_manager::{PLine, ProcessExecution};

// Animated running string based on time
// Has to update every 100 miliseconds
fn running_string() -> String {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let running = match time % 250 {
        0..=100 => "/",
        101..=200 => "-",
        201..=300 => "\\",
        _ => "|",
    };
    format!(" {} Running {} ", running, running)
}

pub fn render_output(f: &mut Frame, area: &Rect, execution: &mut Option<ProcessExecution>) {
    let status_color = match execution {
        Some(exe) => match exe.child.try_wait() {
            Ok(Some(code)) => match code.code() {
                Some(0) => Color::Green,
                Some(_) => Color::LightRed,
                None => Color::LightYellow,
            },
            Ok(None) => Color::LightYellow,
            Err(_) => Color::Red,
        },
        None => Color::Gray,
    };

    let header = match execution {
        Some(exe) => match exe.child.try_wait() {
            Ok(Some(code)) => match code.code() {
                Some(0) => " Success ".to_string(),
                Some(e) => format!(" Error: {} ", e),
                None => running_string(),
            },
            Ok(None) => running_string(),
            Err(_) => " Error ".to_string(),
        },
        None => "No command".to_string(),
    };

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(status_color))
        .border_type(ratatui::widgets::BorderType::Double)
        .title(header)
        .title_alignment(Alignment::Center)
        // .bg(Color::Rgb(30, 34, 42))
        .title_style(Style::default().fg(Color::Yellow).bold());
    f.render_widget(header_block, *area);

    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    // force clear inner_area

    match execution {
        Some(ref mut exe) => {
            let out = exe.output.lock().unwrap();
            // Take bytes until 10 occurences of newline
            let mut last_10: Vec<String> = out
                .split(|&x| x == b'\n')
                .rev()
                .filter(|x| x.len() > 0)
                .take(inner_area.height as usize)
                .map(|x| String::from_utf8_lossy(x))
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            last_10.reverse();

            // eprintln!("Last 10: {:?}", last_10);

            let paragraph = Paragraph::new(Text::from_iter(last_10));
            f.render_widget(paragraph, inner_area);
        }
        None => {
            let paragraph = Paragraph::new(vec![ratatui::text::Line::from("No command running")]);
            f.render_widget(paragraph, inner_area);
        }
    }
}
