use std::io::stdin;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct StdinHandler {
    receiver: mpsc::Receiver<String>,
    sender: mpsc::Sender<String>,
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

    pub fn init(&self) {
        let sender = self.sender.clone();
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
                    Err(_) => todo!(),
                }
            }
        });
    }

    pub fn recv(&self) -> Result<String, mpsc::RecvError> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Result<String, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}
