use is_terminal::IsTerminal;
use logss::app::{App, AppResult};
use logss::args::parse_args;
use logss::event::{Event, EventHandler};
use logss::handler::handle_key_events;
use logss::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

fn main() -> AppResult<()> {
    if std::io::stdin().is_terminal() {
        eprintln!("Please pipe some data to this command. Exiting.");
        std::process::exit(1);
    }
    // Create an application.
    let args = parse_args();
    let mut app = App::new(Some(args));

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let args = parse_args();
    let events = EventHandler::new(args.render);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    // TODO: make it fail and propagate
    app.init();

    // Start the main loop.
    while app.is_running() {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    // Unit test for main function
    #[test]
    fn test_main() {
        let result = main();
        assert!(result.is_ok());
    }
}
