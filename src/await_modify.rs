use std::path::Path;

use crossbeam::channel::{bounded, Receiver};
use notify::{Event, RecommendedWatcher, Watcher};

pub struct ModificationAwaiter {
    _watcher: RecommendedWatcher,
    pub rx: Receiver<Event>,
}

impl ModificationAwaiter {
    pub fn new(p: &Path) -> Self {
        let (tx, rx) = bounded(0);
        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                Ok(e) => {
                    if let notify::EventKind::Modify(_) = e.kind {
                        let _ = tx.try_send(e);
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
}
