use crate::app::{App, AppResult, Views};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // exit application on ESC
        KeyCode::Esc => {
            app.running = false;
        }
        // exit application on Ctrl-D
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            }
        }
        KeyCode::Char('*') => {
            if app.show == Views::RawBuffer {
                app.show = Views::Containers;
            } else {
                app.show = Views::RawBuffer;
            }
        }
        KeyCode::Char('h') => app.help = !app.help,
        KeyCode::Char('w') => app.wrap = !app.wrap,
        _ => {}
    }
    Ok(())
}
