use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if app.show_input() {
        app.update_input(key_event.code);
    } else {
        match key_event.code {
            // exit application on ESC
            KeyCode::Esc => app.stop(),
            // exit application on Ctrl-D
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.stop();
                }
            }
            KeyCode::Char('*') => app.flip_raw_view(),
            KeyCode::Char('i') => app.flip_show_input(),
            KeyCode::Char('h') => app.flip_help(),
            KeyCode::Char('w') => app.flip_wrap(),
            KeyCode::Char('p') => app.flip_pause(),
            KeyCode::Char('v') => app.flip_direction(),
            KeyCode::Char('0') => view_helper(app, 0, key_event),
            KeyCode::Char('1') => view_helper(app, 1, key_event),
            KeyCode::Char('2') => view_helper(app, 2, key_event),
            KeyCode::Char('3') => view_helper(app, 3, key_event),
            KeyCode::Char('4') => view_helper(app, 4, key_event),
            KeyCode::Char('5') => view_helper(app, 5, key_event),
            KeyCode::Char('6') => view_helper(app, 6, key_event),
            KeyCode::Char('7') => view_helper(app, 7, key_event),
            KeyCode::Char('8') => view_helper(app, 8, key_event),
            KeyCode::Char('9') => view_helper(app, 9, key_event),
            KeyCode::Up => {
                app.pause();
                if key_event.kind == KeyEventKind::Press {
                    app.scroll_up();
                }
            }
            KeyCode::Down => {
                app.pause();
                if key_event.kind == KeyEventKind::Press {
                    app.scroll_down();
                }
            }
            KeyCode::Char('c') => {
                app.unpause();
                app.reset_scroll_down();
                app.reset_scroll_up();
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.stop();
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn view_helper(app: &mut App, id: u8, key_event: KeyEvent) {
    if key_event.modifiers == KeyModifiers::ALT {
        app.remove_view(id);
    } else {
        app.zoom_into(id);
    }
}
