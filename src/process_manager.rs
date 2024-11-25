use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Error, ErrorKind, Read},
    process::{Child, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use std::io::Write;

use crossbeam::channel::Receiver;
use tempfile::NamedTempFile;

pub struct ProcessManager {
    pub current_process: Option<Result<ProcessExecution, Error>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            current_process: None,
        }
    }

    pub fn start_p(&mut self, command: String) {
        match ProcessExecution::start_new(command) {
            Ok(p) => self.current_process = Some(Ok(p)),
            Err(e) => self.current_process = Some(Err(e)),
        }
    }
}

pub struct ProcessExecution {
    pub pid: u32,
    pub rx_output: Receiver<String>,
    pub stored_outputs: VecDeque<String>,
    pub child: Child,
}

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
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let pid = child.id();

        let (tx, rx) = crossbeam::channel::unbounded();

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
                    tx.send(line_str).unwrap();
                    line.clear();
                }
            });
        }

        Ok(ProcessExecution {
            pid,
            rx_output: rx,
            stored_outputs: VecDeque::new(),
            child,
        })
    }
}

pub struct ProcessResult {
    pub end_time: std::time::Instant,
    pub return_code: i32,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_process_manager() {
        let mut pm = super::ProcessManager::new();
        pm.start_p("ls".to_string());
        let p = pm.current_process.unwrap();

        assert!(p.is_ok());
    }

    #[test]
    fn test_process_output() {
        let mut pm = super::ProcessManager::new();
        pm.start_p("ls *Cargo*".to_string());
        let mut p = pm.current_process.unwrap().unwrap();
        p.child.wait().unwrap();
        let output = p.rx_output.recv().unwrap();
        assert_eq!(output, "Cargo.lock\n");
        let output = p.rx_output.recv().unwrap();
        assert_eq!(output, "Cargo.toml\n");
    }

    #[test]
    fn test_process_failure() {
        let mut pm = super::ProcessManager::new();
        pm.start_p("not found command".to_string());
        match pm.current_process.unwrap() {
            Ok(_) => panic!("Expected error"),
            Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        }
    }
}
