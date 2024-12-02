use std::{process::exit, thread};
mod await_modify;
mod process_manager;
mod ui;

use await_modify::ModificationAwaiter;
use process_manager::ProcessExecution;

fn main() {
    let p = std::env::current_dir().unwrap();
    let awaiter = ModificationAwaiter::new(p.as_path());
    let command = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let (tx, rx) = crossbeam::channel::unbounded();
    match ProcessExecution::start_new(command.clone()) {
        Ok(process_exe) => {
            tx.send(process_exe).unwrap();
        }
        Err(e) => {
            eprintln!("Was not able to start with command: {:?}", command);
            eprintln!("{:?}", e);
            exit(1);
        }
    }

    thread::spawn(|| {
        ui::init(rx).unwrap();
        exit(0);
    });

    loop {
        let event = awaiter.await_modify();
        match event {
            Ok(_) => {
                match ProcessExecution::start_new(command.clone()) {
                    Ok(process_exe) => {
                        tx.send(process_exe).unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        exit(1);
                    }
                }
                // println!("{:?}", event);
            }
            Err(_) => {
                // println!("Error: {:?}", e);
            }
        }
    }
}
