use clap::Parser;
use is_terminal::IsTerminal;
use logss::app::{App, AppResult};
use logss::args::Args;
use logss::event::{Event, EventHandler};
use logss::handler::handle_key_events;
use logss::tui::Tui;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> AppResult<()> {
    if !std::io::stdout().is_terminal() {
        println!("This is not a terminal. Exiting.");
        return Ok(());
    }
    // Create an application.
    let args = Args::parse();
    let mut app = App::new(Some(args));

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let args = Args::parse();
    let events = EventHandler::new(args.render);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
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
