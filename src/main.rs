use std::{path::Path, thread};
mod await_modify;
mod process_manager;
mod ui;

use await_modify::ModificationAwaiter;
use process_manager::ProcessManager;

fn main() {
    let p = std::env::current_dir().unwrap();
    let mut pm = ProcessManager::new();
    let command = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    pm.start_p(command);
    // pm.start_p("ls".to_string());

    // get current directory full path
    // eprintln!("Watching {:?}", &p);
    // let awaiter = ModificationAwaiter::new(&p);

    ui::init(&mut pm).unwrap();
    // thread::spawn(|| {
    // });

    // loop {
    //     let event = awaiter.await_modify();
    //     match event {
    //         Ok(event) => {
    //             println!("{:?}", event);
    //         }
    //         Err(e) => {
    //             println!("Error: {:?}", e);
    //         }
    //     }
    // }
}
