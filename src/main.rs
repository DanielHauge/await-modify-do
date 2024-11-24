use std::{path::Path, thread};
mod await_modify;
mod ui;

use await_modify::ModificationAwaiter;

fn main() {
    let p = std::env::current_dir().unwrap();
    // get current directory full path
    // eprintln!("Watching {:?}", &p);
    // let awaiter = ModificationAwaiter::new(&p);

    ui::init().unwrap();
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
