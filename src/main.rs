use std::{process::exit, thread};
mod await_modify;
mod process_manager;
mod ui;

use await_modify::ModificationAwaiter;
use process_manager::ProcessExecution;

fn main() {
    let p = std::env::current_dir().expect("Could not get current directory");
    let awaiter = ModificationAwaiter::new(p.as_path());
    let command = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let (tx, rx) = crossbeam::channel::unbounded();
    let launcher = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    match ProcessExecution::start_new(&launcher, command.clone()) {
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

    thread::spawn(|| match ui::init(rx) {
        Ok(_) => exit(0),
        Err(e) => {
            eprintln!("Exited abnormally because of error: {:?}", e);
            exit(1);
        }
    });

    loop {
        let event = awaiter.await_modify();
        match event {
            Ok(_) => match ProcessExecution::start_new(&launcher, command.clone()) {
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
