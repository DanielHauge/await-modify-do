use std::{
    io::{stdout, Result},
    rc::Rc,
};

use header::render_header;
use ratatui::{
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Alignment, Constraint, Flex, Layout, Position, Rect},
    prelude::CrosstermBackend,
    style::Stylize,
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

mod header;

// Make 3 panels, a header, a sub header, and a main panel that takes up the rest of the screen
// The header should be five lines, the sub header should be 4 line, and the main panel
// should be the rest of the screen
// The header should be red, the sub header should be blue, and the main panel should be green
// The header should say "Header", the sub header should say "Sub Header", and the main panel
// should say "Main Panel"
fn make_panels_rect(area: Rect) -> Rc<[Rect]> {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5),
                Constraint::Length(4),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);
    chunks
}

pub fn init() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // State -> Move to eventual state module
    // let mut cursor = Position::new(1, 1);

    loop {
        // Rendering using state -> Move to eventual ui module
        terminal.draw(|frame| {
            // frame.set_cursor_position(cursor);
            let areas = make_panels_rect(frame.area());
            let [header_area, stats_area, output_area] = areas.as_ref() else {
                todo!()
            };
            render_header(frame, header_area);
        })?;

        // Interaction to modify state -> Move to eventual ux module
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match (key.kind, key.code) {
                    (KeyEventKind::Press, KeyCode::Char('q')) => break,
                    _ => {}
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
