use crate::app::AppResult;

use std::io::stdin;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct StdinHandler {
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
    handler: thread::JoinHandle<()>,
}

impl StdinHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let stdin = stdin();
                loop {
                    let mut line = String::new();
                    match stdin.read_line(&mut line) {
                        Ok(len) => {
                            if len == 0 {
                                break;
                            } else {
                                sender.send(line).unwrap();
                            }
                        }
                        Err(_) => todo!(),
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn recv(&self) -> AppResult<String> {
        Ok(self.receiver.recv()?)
    }

    pub fn try_recv(&self) -> AppResult<String> {
        Ok(self.receiver.try_recv()?)
    }
}
