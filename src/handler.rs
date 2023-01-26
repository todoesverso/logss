use crate::app::{App, AppResult, Views};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

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
        KeyCode::Char('p') => app.pause = !app.pause,
        KeyCode::Char('0') => {
            zoom_view_helper(app, 0);
        }
        KeyCode::Char('1') => {
            zoom_view_helper(app, 1);
        }
        KeyCode::Char('2') => {
            zoom_view_helper(app, 2);
        }
        KeyCode::Char('3') => {
            zoom_view_helper(app, 3);
        }
        KeyCode::Char('4') => {
            zoom_view_helper(app, 4);
        }
        KeyCode::Char('5') => {
            zoom_view_helper(app, 5);
        }
        KeyCode::Char('6') => {
            zoom_view_helper(app, 6);
        }
        KeyCode::Char('7') => {
            zoom_view_helper(app, 7);
        }
        KeyCode::Char('8') => {
            zoom_view_helper(app, 8);
        }
        KeyCode::Char('9') => {
            zoom_view_helper(app, 9);
        }
        KeyCode::Up => {
            app.pause = true;
            if key_event.kind == KeyEventKind::Press {
                app.up += 1;
            }
        }
        KeyCode::Down => {
            app.pause = true;
            if key_event.kind == KeyEventKind::Press {
                app.down += 1;
            }
        }
        KeyCode::Char('c') => {
            app.pause = false;
            app.down = 0;
            app.up = 0;
        }
        _ => {}
    }
    Ok(())
}

fn zoom_view_helper(app: &mut App, id: u8) {
    if !app.containers.values().map(|c| c.id).any(|x| x == id) {
        return;
    }

    if app.show == Views::Zoom {
        app.show = Views::Containers;
        app.zoom_id = None;
    } else {
        app.show = Views::Zoom;
        app.zoom_id = Some(id);
    }
}
