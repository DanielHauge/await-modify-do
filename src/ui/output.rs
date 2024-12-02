use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Stylize},
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
        .bg(Color::Rgb(30, 34, 42))
        .title_style(Style::default().fg(Color::Yellow).bold());
    f.render_widget(header_block, *area);

    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    match execution {
        Some(ref mut exe) => {
            let mut max_lines_output = 100;
            loop {
                if max_lines_output == 0 {
                    break;
                }
                let mut had_output = false;
                if let Ok(output) = exe.rx_err.try_recv() {
                    exe.stored_outputs.push_back(output);
                    max_lines_output -= 1;
                    had_output = true;
                }
                if let Ok(output) = exe.rx_output.try_recv() {
                    exe.stored_outputs.push_back(output);
                    max_lines_output -= 1;
                    had_output = true;
                };
                if !had_output {
                    break;
                }
            }
            let last_10 = exe.stored_outputs.iter().take(inner_area.height as usize);
            let mut lines = Vec::new();
            for pline in last_10 {
                let rline = match pline {
                    PLine::Stdout(output) => ratatui::text::Line::from(output.as_str()),
                    PLine::Stderr(output) => ratatui::text::Line::from(output.as_str().red()),
                };
                lines.push(rline);
            }
            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner_area);
            // let output_str = output_str
            //     .lines()
            //     .rev()
            //     .take(10)
            //     .collect::<Vec<&str>>()
            //     .join("\n");
            // let text = Text::from(output_str);
            // let paragraph = Paragraph::new(text);
            // f.render_widget(paragraph, inner_area);
            ()
        }
        None => todo!(),
    }
}
