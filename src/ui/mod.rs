use std::{
    io::{stdout, Result},
    rc::Rc,
};

use crossbeam::channel::{Receiver, Sender};
use header::render_header;
use output::render_output;
use ratatui::{
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    Terminal,
};
use stats::render_stats;
use sysinfo::System;

use crate::process_manager::ProcessExecution;

mod header;
mod output;
mod stats;

fn make_panels_rect(area: Rect) -> Rc<[Rect]> {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);
    chunks
}

pub fn init(rx_pm: Receiver<ProcessExecution>, manual_trigger_tx: Sender<()>) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut currrent_execution: Option<ProcessExecution> = None;
    let mut system = System::new_all();
    let current_pid = sysinfo::get_current_pid().expect("Could not get current pid");
    let parrent_pid = system
        .process(current_pid)
        .expect("Could not find information for current pid")
        .parent()
        .expect("Current program doesn't have a parent process, which it is designed to have using the shell");

    loop {
        if let Ok(execution) = rx_pm.try_recv() {
            if let Some(mut exe) = currrent_execution {
                exe.child.kill().expect("Could not kill the process");
            }
            currrent_execution = Some(execution);
        }
        terminal.draw(|frame| {
            let areas = make_panels_rect(frame.area());
            let [header_area, stats_area, output_area] = areas.as_ref() else {
                panic!("Could not get the areas for the panels");
            };
            render_header(frame, header_area);
            render_stats(
                frame,
                stats_area,
                &mut system,
                &mut currrent_execution,
                &parrent_pid,
            );
            render_output(frame, output_area, &mut currrent_execution);
        })?;

        // Interaction to modify state -> Move to eventual ux module
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if let (KeyEventKind::Press, KeyCode::Char('q')) = (key.kind, key.code) {
                    if let Some(ref mut exe) = currrent_execution {
                        exe.child.kill().expect("Could not kill the process");
                    }
                    break;
                }
                // If escape is pressed, kill the current process
                if let (KeyEventKind::Press, KeyCode::Esc) = (key.kind, key.code) {
                    if let Some(ref mut exe) = currrent_execution {
                        if exe
                            .child
                            .try_wait()
                            .expect("Could not get the status of the process")
                            .is_none()
                        {
                            exe.child.kill().expect("Could not kill the process");
                            exe.cancelled = true;
                        }
                    }
                }
                // If space rerun
                if let (KeyEventKind::Press, KeyCode::Char(' ')) = (key.kind, key.code) {
                    if let Some(ref mut exe) = currrent_execution {
                        exe.child.kill().expect("Could not kill the process");
                        exe.cancelled = true;
                    }
                    manual_trigger_tx
                        .send(())
                        .expect("Could not send manual trigger");
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    if let Some(mut exe) = currrent_execution {
        exe.child.kill().expect("Could not kill the process");
    }
    Ok(())
}
