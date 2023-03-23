use crate::app::AppResult;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Terminal events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    tick_rate: u64,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            receiver,
            sender,
            tick_rate,
        }
    }

    pub fn init(&self) {
        let tick_rate = Duration::from_millis(self.tick_rate);
        let sender = self.sender.clone();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(tick_rate);

                if event::poll(timeout).expect("no events available") {
                    match event::read().expect("unable to read event") {
                        CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                        CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                        CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                        _ => Ok(()),
                    }
                    .expect("failed to send terminal event")
                }

                if last_tick.elapsed() >= tick_rate {
                    sender.send(Event::Tick).ok();
                    last_tick = Instant::now();
                }
            }
        });
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    #[test]
    fn new() {
        let event = EventHandler::new(1);
        event.sender.send(Event::Tick).unwrap();
        assert_eq!(event.next().unwrap(), Event::Tick);
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        event.sender.send(Event::Key(key)).unwrap();
        assert_eq!(event.next().unwrap(), Event::Key(key));
    }
    #[test]
    fn init() {
        // just call it and expect not to panic
        let event = EventHandler::new(1);
        event.init();
    }
}
