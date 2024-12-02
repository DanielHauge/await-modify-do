use core::panic;
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{Child, Command, Stdio},
    thread,
};

use crossbeam::channel::Receiver;

#[derive(Debug, PartialEq)]
pub enum PLine {
    Stdout(String),
    Stderr(String),
}

pub struct ProcessExecution {
    pub rx_output: Receiver<PLine>,
    pub rx_err: Receiver<PLine>,
    pub stored_outputs: VecDeque<PLine>,
    pub child: Child,
}

pub const ENVVAR_BACKTRACE: &str = "AWAIT_MODIFY_DO_BACKTRACE";

impl ProcessExecution {
    pub fn start_new(commandline: String) -> Result<ProcessExecution, Error> {
        let command_split = commandline.split_whitespace().collect::<Vec<&str>>();
        if command_split.len() == 0 {
            return Err(Error::new(ErrorKind::Other, "No command provided"));
        }
        let command = command_split[0].to_string();
        let args = command_split[1..].to_vec();
        let mut child = Command::new(&command.trim().to_string())
            .args(args)
            .env(ENVVAR_BACKTRACE, "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // .stdin(Stdioinheriterit)
            .spawn()?;

        let (tx_output, rx_output) = crossbeam::channel::unbounded();
        let (tx_err, rx_err) = crossbeam::channel::unbounded();

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            thread::spawn(move || {
                while let Ok(bytes_read) = reader.read_line(&mut line) {
                    if bytes_read == 0 {
                        break; // EOF reached
                    }
                    // Write to the temp file
                    let line_str = line.clone();
                    match tx_output.send(PLine::Stdout(line_str)) {
                        Ok(_) => {}
                        Err(x) => panic!("Error: {:?}", x.0),
                    }
                    line.clear();
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            thread::spawn(move || {
                while let Ok(bytes_read) = reader.read_line(&mut line) {
                    if bytes_read == 0 {
                        break; // EOF reached
                    }
                    // Write to the temp file
                    let line_str = line.clone();
                    match tx_err.send(PLine::Stderr(line_str)) {
                        Ok(_) => {}
                        Err(x) => panic!("Error: {:?}", x.0),
                    }
                    line.clear();
                }
            });
        }

        Ok(ProcessExecution {
            rx_output,
            rx_err,
            stored_outputs: VecDeque::new(),
            child,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::process_manager::{PLine, ProcessExecution};

    #[test]
    fn test_process_output() {
        let mut p = ProcessExecution::start_new("ls *Cargo*".to_string()).unwrap();
        p.child.wait().unwrap();
        let output = p.rx_output.recv().unwrap();
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
