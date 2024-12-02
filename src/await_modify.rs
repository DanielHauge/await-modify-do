use std::path::Path;

use crossbeam::channel::{bounded, Receiver, RecvError};
use notify::{Event, RecommendedWatcher, Watcher};

pub struct ModificationAwaiter {
    _watcher: RecommendedWatcher,
    rx: Receiver<Event>,
}

impl ModificationAwaiter {
    pub fn new(p: &Path) -> Self {
        let (tx, rx) = bounded(10);
        let mut last_trigger = std::time::Instant::now();
        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(e) => {
                if last_trigger.elapsed().as_millis() > 3001 {
                    match tx.try_send(e) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                    last_trigger = std::time::Instant::now();
                }
            }
            Err(x) => panic!("watch error: {:?}", x),
        })
        .unwrap();
        watcher.watch(p, notify::RecursiveMode::Recursive).unwrap();
        Self {
            _watcher: watcher,
            rx,
        }
    }

    pub fn await_modify(&self) -> Result<Event, RecvError> {
        self.rx.recv()
    }
}
