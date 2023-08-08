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
    let args = parse_args();
    let render_speed = args.render.unwrap_or(100);

    if args.command.is_none() && std::io::stdin().is_terminal() {
        eprintln!("No command provided and no data piped.");
        eprintln!("Please pipe some data to this command. Exiting.");
        std::process::exit(1);
    }

    // Create an application.
    let mut app = App::new(Some(args));
    // First we try to start app so that it can fail and we do not mess with the console
    app.init()?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(render_speed);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

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
    //use super::*;

    // Unit test for main function
    #[test]
    fn test_main() {
        //let result = main();
        //assert!(result.is_ok());
    }
}
