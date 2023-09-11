use std::io::{stdin, Error, ErrorKind};
use std::sync::mpsc;
use std::thread;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::app::AppResult;

#[derive(Debug)]
pub struct StdinHandler {
    receiver: mpsc::Receiver<String>,
    pub sender: mpsc::Sender<String>,
}

impl Default for StdinHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl StdinHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { receiver, sender }
    }

    pub fn init(&self, cmd: Option<Vec<String>>) -> AppResult<()> {
        // TODO: refactor to avoid duplicate code
        let sender = self.sender.clone();
        if let Some(inner_cmd) = cmd {
            let child = Command::new(&inner_cmd[0])
                .args(&inner_cmd[1..])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;

            let stdout = child
                .stdout
                .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to run command"))?;

            let mut reader = BufReader::new(stdout);

            thread::spawn(move || loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(len) => {
                        if len == 0 {
                            break;
                        } else {
                            sender.send(line).ok();
                        }
                    }
                    Err(_) => panic!("BUG!, please report it"),
                }
            });
        } else {
            let reader = stdin();

            thread::spawn(move || loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(len) => {
                        if len == 0 {
                            break;
                        } else {
                            sender.send(line).ok();
                        }
                    }
                    Err(_) => panic!("BUG!, please report it"),
                }
            });
        }

        Ok(())
    }

    pub fn recv(&self) -> Result<String, mpsc::RecvError> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Result<String, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}
