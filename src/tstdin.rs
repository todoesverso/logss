use std::io::stdin;
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
        let sender = self.sender.clone();
        if let Some(inner_cmd) = cmd {
            let child = Command::new(&inner_cmd[0])
                .args(&inner_cmd[1..])
                .stdout(Stdio::piped())
                .spawn()?;
            //child.stderr.take();

            let stdout = child
                .stdout
                .ok_or_else(|| panic!("Failed to run command"))
                .unwrap();
            let mut reader = BufReader::new(stdout);
            loop {
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
            }
        } else {
            thread::spawn(move || {
                let stdin = stdin();
                loop {
                    let mut line = String::new();
                    match stdin.read_line(&mut line) {
                        Ok(len) => {
                            if len == 0 {
                                break;
                            } else {
                                sender.send(line).ok();
                            }
                        }
                        Err(_) => panic!("BUG!, please report it"),
                    }
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
