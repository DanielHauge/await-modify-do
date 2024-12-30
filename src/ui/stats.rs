use std::collections::HashSet;

use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, LineGauge},
    Frame,
};
use sysinfo::{Gid, Pid, System};

use crate::process_manager::ProcessExecution;

pub fn render_stats(
    f: &mut Frame,
    area: &Rect,
    system: &mut System,
    execution: &mut Option<ProcessExecution>,
    _current_pid: &Pid,
) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightGreen))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title("Stats")
        .title_alignment(Alignment::Center)
        // .bg(Color::Rgb(30, 34, 42))
        .title_style(Style::default().fg(Color::Yellow).bold());
    f.render_widget(header_block, *area);

    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let (total_cpu_area, total_memory_area, total_swap_area) = split_row_layout(inner_area);

    match execution {
        Some(_) => {
            let current_time_in_seconds = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            // If time modlus 1000 is less than 100, then update
            if current_time_in_seconds % 1000 < 40 {
                system.refresh_all();
            }
            let total_cpu_usage = system.global_cpu_usage();
            let total_memory = human_friendly_mem(system.total_memory());
            let total_used_memory = human_friendly_mem(system.used_memory());
            let total_swap = human_friendly_mem(system.total_swap());
            let total_used_swap = human_friendly_mem(system.used_swap());
            // let exit = match e.child.try_wait() {
            //     Ok(x) => x.map(|e| e.code()),
            //     Err(_) => None,
            // };

            // CPU
            let cpu_str = "CPU: ";
            let (cpu_txt_area, cpu_gauge_area) =
                split_line_layout(total_cpu_area, (cpu_str).len() as u16);
            f.render_widget(
                Text::from(cpu_str).style(Style::default().fg(Color::Yellow)),
                cpu_txt_area,
            );
            f.render_widget(
                LineGauge::default()
                    .filled_style(Style::default().fg(Color::Cyan))
                    .ratio(total_cpu_usage as f64 / 100.0),
                cpu_gauge_area,
            );

            // Memory
            let mem_label = Span::from("Memory: ").style(Style::default().fg(Color::Yellow));
            let mem_str = Span::from(format!("{} / {} - ", total_used_memory, total_memory));
            let total_memory_str = Text::from(Line::from(vec![mem_label, mem_str]));
            let (mem_txt_area, mem_guage_area) =
                split_line_layout(total_memory_area, total_memory_str.width() as u16);
            f.render_widget(total_memory_str, mem_txt_area);
            let mem_color = if system.used_memory() as f64 / system.total_memory() as f64 > 0.8 {
                Color::Red
            } else {
                Color::Cyan
            };
            f.render_widget(
                LineGauge::default()
                    .filled_style(Style::default().fg(mem_color))
                    .ratio(system.used_memory() as f64 / system.total_memory() as f64),
                mem_guage_area,
            );

            // Swap
            let swap_label = Span::from("Swap: ").style(Style::default().fg(Color::Yellow));
            let swap_str = Span::from(format!("{} / {} - ", total_used_swap, total_swap));
            let total_swap_str = Text::from(Line::from(vec![swap_label, swap_str]));
            let (swap_txt_area, swap_gauge_area) =
                split_line_layout(total_swap_area, total_swap_str.width() as u16);
            f.render_widget(total_swap_str, swap_txt_area);
            let swap_color = if system.used_swap() as f64 / system.total_swap() as f64 > 0.8 {
                Color::Red
            } else {
                Color::Cyan
            };
            f.render_widget(
                LineGauge::default()
                    .filled_style(Style::default().fg(swap_color))
                    .ratio(system.used_swap() as f64 / system.total_swap() as f64),
                swap_gauge_area,
            );

            // let text = Text::raw(lines.join("\n"));
            // f.render_widget(text, inner_area);
        }
        None => panic!("Nothing is running"),
    }
}

fn split_row_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Max(1), Constraint::Max(1), Constraint::Max(1)].as_ref())
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

fn split_line_layout<T: Into<u16>>(area: Rect, len: T) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Length(len.into()), Constraint::Min(0)].as_ref())
        .split(area);
    (chunks[0], chunks[1])
}

fn human_friendly_mem(bytes: u64) -> String {
    let bytes_f = bytes as f64;
    let kb = bytes_f / 1024.0;
    let mb = kb / 1024.0;
    let gb = mb / 1024.0;
    let tb = gb / 1024.0;

    if tb > 1.0 {
        format!("{:.2} TB", tb)
    } else if gb > 1.0 {
        format!("{:.2} GB", gb)
    } else if mb > 1.0 {
        format!("{:.2} MB", mb)
    } else if kb > 1.0 {
        format!("{:.2} KB", kb)
    } else {
        format!("{:.2} B", bytes)
    }
}

#[allow(dead_code)]
fn get_all_tracked(system: &mut System) -> Vec<Vec<String>> {
    system
        .processes()
        .iter()
        .map(|(_, proc)| {
            proc.environ()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

#[allow(dead_code)]
fn get_all_in_group(system: &mut System, own_pid: Gid) -> HashSet<Pid> {
    let mut all_pids_in_group = HashSet::new();
    for (pid, process) in system.processes() {
        if process.effective_group_id() == Some(own_pid) {
            all_pids_in_group.insert(*pid);
        }
    }
    all_pids_in_group
}

#[allow(dead_code)]
fn get_all_session_processes(system: &mut System, session_pid: Pid) -> HashSet<Pid> {
    let mut all_pids_in_group = HashSet::new();
    for (pid, process) in system.processes() {
        if process.session_id() == Some(session_pid) {
            all_pids_in_group.insert(*pid);
        }
    }
    all_pids_in_group
}

#[allow(dead_code)]
fn get_all_children(system: &System, pid: Pid) -> HashSet<Pid> {
    let mut children = HashSet::new();

    for (proc_pid, process) in system.processes() {
        if process.parent() == Some(pid) {
            // Add the child PID
            children.insert(*proc_pid);
            // Recursively find its children
            children.extend(get_all_children(system, *proc_pid));
        }
    }

    children
}
