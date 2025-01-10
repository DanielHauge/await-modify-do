use core::panic;
use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Error, ErrorKind, Read},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::Receiver;
use ratatui::crossterm;

#[derive(Debug, PartialEq)]
pub enum PLine {
    Stdout(String),
    Stderr(String),
}

pub struct ProcessExecution {
    pub output: Arc<Mutex<Vec<u8>>>,
    pub child: Child,
}

impl ProcessExecution {
    pub fn start_new(launcher: &str, commandline: String) -> Result<ProcessExecution, Error> {
        let command_split = commandline.split_whitespace().collect::<Vec<&str>>();
        if command_split.len() == 0 {
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
            thread::spawn(move || {
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 {
                        break; // EOF reached
                    }
                    // Write to the temp file
                    let chunk = &buffer[..bytes_read];
                    output_clone.lock().unwrap().extend_from_slice(chunk);
                    // unlock the mutex
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr);
            let mut buffer = [0; 1024];
            thread::spawn(move || {
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 {
                        break; // EOF reached
                    }
                    let chunk = &buffer[..bytes_read];
                    err_clone.lock().unwrap().extend_from_slice(chunk);
                }
            });
        }

        Ok(ProcessExecution { output, child })
    }
}

#[cfg(test)]
mod tests {
    use crate::process_manager::{PLine, ProcessExecution};

    #[test]
    fn test_process_output() {
        let mut p = ProcessExecution::start_new("ls".to_string()).unwrap();
        let output = match p.rx_output.recv() {
            Ok(o) => o,
            Err(e) => panic!("Error: {:?}", e),
        };
        p.child.wait().unwrap();
        if let PLine::Stdout(output) = output {
            assert_eq!(output, "Cargo.lock\n");
        } else {
            panic!("Expected stdout");
        }
        if let PLine::Stdout(output) = p.rx_output.recv().unwrap() {
            assert_eq!(output, "Cargo.toml\n");
        } else {
            panic!("Expected end");
        }
    }

    #[test]
    fn test_process_failure() {
        let p = ProcessExecution::start_new("somebscommand".to_string());
        match p {
            Ok(_) => panic!("Expected error"),
            Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        }
    }
}
