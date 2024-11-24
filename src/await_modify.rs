use std::path::Path;

use crossbeam::channel::{bounded, Receiver, RecvError};
use notify::{Event, ReadDirectoryChangesWatcher, Watcher};

pub struct ModificationAwaiter {
    watcher: ReadDirectoryChangesWatcher,
    rx: Receiver<Event>,
}

impl ModificationAwaiter {
    pub fn new(p: &Path) -> Self {
        let (tx, rx) = bounded(30);
        let mut last_trigger = std::time::Instant::now();
        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(e) => {
                if last_trigger.elapsed().as_millis() > 500 {
                    tx.send(e).unwrap();
                    last_trigger = std::time::Instant::now();
                }
            }
            Err(x) => panic!("watch error: {:?}", x),
        })
        .unwrap();
        watcher.watch(p, notify::RecursiveMode::Recursive).unwrap();
        Self { watcher, rx }
    }

    pub fn await_modify(&self) -> Result<Event, RecvError> {
        self.rx.recv()
    }
}
