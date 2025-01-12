use std::{process::exit, thread};
mod await_modify;
mod process_manager;
mod ui;

use await_modify::ModificationAwaiter;
use crossbeam::{
    channel::{self, unbounded, Receiver},
    select,
};
use process_manager::{EndType, ProcessExecution, Trigger};

fn main() {
    let p = std::env::current_dir().expect("Could not get current directory");
    let awaiter = ModificationAwaiter::new(p.as_path());
    let command = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let (tx, rx) = unbounded();
    let (manual_trigger_tx, manual_trigger_rx) = channel::bounded(0);
    let (end_tx, end_rx) = unbounded();
    let launcher = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    match ProcessExecution::start_new(&launcher, command.clone(), &end_tx, Trigger::Start) {
        Ok(process_exe) => {
            tx.send(process_exe)
                .expect("Could not send process execution start to unbounded channel");
        }
        Err(e) => {
            eprintln!("Was not able to start with command: {:?}", command);
            eprintln!("{:?}", e);
            exit(1);
        }
    }

    thread::spawn(|| match ui::init(rx, manual_trigger_tx) {
        Ok(_) => exit(0),
        Err(e) => {
            eprintln!("Exited abnormally because of error: {:?}", e);
            exit(1);
        }
    });

    loop {
        await_end(&end_rx);
        select! {
            recv(manual_trigger_rx) -> _ => {
                match ProcessExecution::start_new(&launcher, command.clone(), &end_tx, Trigger::Manual) {
                    Ok(process_exe) => {
                        tx.send(process_exe)
                            .expect("Could not send process execution start to unbounded channel");
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        exit(1);
                    }
                }
            }
            recv(awaiter.rx) -> event => {
                match event {
                    Ok(e) => match ProcessExecution::start_new(
                        &launcher,
                        command.clone(),
                        &end_tx,
                        Trigger::Modify(
                            e.paths
                                .first()
                                .unwrap()
                                .to_path_buf()
                                .to_str()
                                .unwrap()
                                .to_string(),
                        ),
                    ) {
                        Ok(process_exe) => {
                            tx.send(process_exe)
                                .expect("Could not send process execution start to unbounded channel");
                        }
                        Err(e) => {
                            eprintln!("Error: {:?}", e);
                            exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!("Could not rerun because: {:?}", e);
                        exit(1);
                    }
                }
            }
        }
    }
}

fn await_end(end_rx: &Receiver<EndType>) {
    let first_end = end_rx.recv().expect("Could not receive from channel");
    let second_end = end_rx.recv().expect("Could not receive from channel");
    match (first_end, second_end) {
        (EndType::Stdout, EndType::Stderr) => {}
        (EndType::Stderr, EndType::Stdout) => {}
        (a, b) => {
            eprintln!(
                "Error: Could not receive both stdout and stderr, got {:?} and {:?}",
                a, b
            );
            exit(1);
        }
    }
}
