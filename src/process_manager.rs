use std::{
    io::{BufReader, Error, ErrorKind, Read},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::Sender;

pub enum Trigger {
    Modify(String),
    Manual,
    Start,
}

pub struct ProcessExecution {
    pub output: Arc<Mutex<Vec<u8>>>,
    pub child: Child,
    pub cancelled: bool,
    pub trigger: Trigger,
}

#[derive(Debug)]
pub enum EndType {
    Stdout,
    Stderr,
}

impl ProcessExecution {
    pub fn start_new(
        launcher: &str,
        commandline: String,
        tx_end: &Sender<EndType>,
        trigger: Trigger,
    ) -> Result<ProcessExecution, Error> {
        let command_split = commandline.split_whitespace().collect::<Vec<&str>>();
        if command_split.is_empty() {
            return Err(Error::new(ErrorKind::Other, "No command provided"));
        }

        let mut child = Command::new(launcher)
            .arg("-c")
            .arg(commandline)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = output.clone();
        let err_clone = output.clone();

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut buffer = [0; 1024];
            let tx_end = tx_end.clone();
            thread::spawn(move || {
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 {
                        break; // EOF reached
                    }
                    let chunk = &buffer[..bytes_read];
                    output_clone.lock().unwrap().extend_from_slice(chunk);
                }
                tx_end.send(EndType::Stdout).unwrap();
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr);
            let mut buffer = [0; 1024];
            let tx_end = tx_end.clone();
            thread::spawn(move || {
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 {
                        break; // EOF reached
                    }
                    let chunk = &buffer[..bytes_read];
                    err_clone.lock().unwrap().extend_from_slice(chunk);
                }
                tx_end.send(EndType::Stderr).unwrap();
            });
        }

        Ok(ProcessExecution {
            output,
            child,
            cancelled: false,
            trigger,
        })
    }
}
