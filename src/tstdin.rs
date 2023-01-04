use std::io::stdin;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct StdinHandler {
    receiver: mpsc::Receiver<String>,
}

impl Default for StdinHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl StdinHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let sender = sender;
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
        });
        Self { receiver }
    }

    pub fn recv(&self) -> Result<String, mpsc::RecvError> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Result<String, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}
