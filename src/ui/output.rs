use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Styled, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::process_manager::ProcessManager;

pub fn render_output(f: &mut Frame, area: &Rect, pm: &mut ProcessManager) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightGreen))
        .border_type(ratatui::widgets::BorderType::Double)
        .title(format!("output {}", 5))
        .title_alignment(Alignment::Center)
        .bg(Color::Rgb(30, 34, 42))
        .title_style(Style::default().fg(Color::Yellow).bold());
    f.render_widget(header_block, *area);

    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    match pm.current_process {
        Some(ref mut p) => match p {
            Ok(ref mut exe) => {
                let mut max_lines_output = 100;
                while let Ok(output) = exe.rx_output.try_recv() {
                    exe.stored_outputs.push_front(output);
                    max_lines_output -= 1;
                    if max_lines_output == 0 {
                        break;
                    }
                }
                let last_10 = exe.stored_outputs.iter().take(inner_area.height as usize);
                let mut lines = Vec::new();
                for line in last_10.rev() {
                    let text = Line::from(line.clone());
                    lines.push(text);
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
            Err(e) => {
                let paragraph = Paragraph::new(e.to_string());
                f.render_widget(paragraph, inner_area);
            }
        },
        None => (),
    }
}
