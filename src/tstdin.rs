use std::io::{stdin, Error, ErrorKind};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
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
                .stderr(Stdio::null())
                .stdout(Stdio::piped())
                .spawn()?;

            let stdout = child
                .stdout
                .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to run command"))?;
            let reader = BufReader::new(stdout);
            read_lines_and_send(reader, sender);
        } else {
            let stdin = stdin();
            let reader = BufReader::new(stdin);

            read_lines_and_send(reader, sender);
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

fn read_lines_and_send<R>(mut reader: R, sender: Sender<String>)
where
    R: BufRead + Send + 'static,
{
    let mut line = String::new();
    thread::spawn(move || loop {
        match reader.read_line(&mut line) {
            Ok(len) => {
                if len == 0 {
                    break;
                } else {
                    sender.send(line.clone()).ok();
                }
            }
            Err(_) => panic!("BUG!, please report it"),
        }
        line.clear();
    });
}
