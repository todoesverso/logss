use crate::app::{App, AppResult};
use crate::states::Views;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui::layout::Direction;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if app.state.show_input {
        match key_event.code {
            KeyCode::Enter => {
                app.add_input_as_container();
                app.state.show_input = false
            }
            KeyCode::Char(c) => {
                app.input.input.push(c);
            }
            KeyCode::Backspace => {
                app.input.input.pop();
            }
            KeyCode::Esc => {
                app.state.show_input = false;
            }
            _ => {}
        }
    } else {
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
                if !app.containers.is_empty() {
                    if app.state.show == Views::RawBuffer {
                        app.state.show = Views::Containers;
                    } else {
                        app.state.show = Views::RawBuffer;
                    }
                }
            }
            KeyCode::Char('i') => app.state.show_input = !app.state.show_input,
            KeyCode::Char('h') => app.state.help = !app.state.help,
            KeyCode::Char('w') => app.state.wrap = !app.state.wrap,
            KeyCode::Char('p') => app.state.paused = !app.state.paused,
            KeyCode::Char('v') => {
                if app.state.direction == Direction::Vertical {
                    app.state.direction = Direction::Horizontal;
                } else {
                    app.state.direction = Direction::Vertical;
                }
            }
            KeyCode::Char('0') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 0);
                } else {
                    zoom_view_helper(app, 0);
                }
            }
            KeyCode::Char('1') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 1);
                } else {
                    zoom_view_helper(app, 1);
                }
            }
            KeyCode::Char('2') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 2);
                } else {
                    zoom_view_helper(app, 2);
                }
            }
            KeyCode::Char('3') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 3);
                } else {
                    zoom_view_helper(app, 3);
                }
            }
            KeyCode::Char('4') => {
                zoom_view_helper(app, 4);
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 4);
                }
            }
            KeyCode::Char('5') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 5);
                } else {
                    zoom_view_helper(app, 5);
                }
            }
            KeyCode::Char('6') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 6);
                } else {
                    zoom_view_helper(app, 6);
                }
            }
            KeyCode::Char('7') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 7);
                } else {
                    zoom_view_helper(app, 7);
                }
            }
            KeyCode::Char('8') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 8);
                } else {
                    zoom_view_helper(app, 8);
                }
            }
            KeyCode::Char('9') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    remove_view_helper(app, 9);
                } else {
                    zoom_view_helper(app, 9);
                }
            }
            KeyCode::Up => {
                app.state.paused = true;
                if key_event.kind == KeyEventKind::Press {
                    app.state.scroll_up += 1;
                }
            }
            KeyCode::Down => {
                app.state.paused = true;
                if key_event.kind == KeyEventKind::Press {
                    app.state.scroll_down += 1;
                }
            }
            KeyCode::Char('c') => {
                app.state.paused = false;
                app.state.scroll_down = 0;
                app.state.scroll_up = 0;
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.running = false;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn zoom_view_helper(app: &mut App, id: u8) {
    if !app.containers.values().map(|c| c.id).any(|x| x == id) {
        return;
    }

    if app.state.show == Views::Zoom {
        app.state.show = Views::Containers;
        app.state.zoom_id = None;
    } else {
        app.state.show = Views::Zoom;
        app.state.zoom_id = Some(id);
    }
}

fn remove_view_helper(app: &mut App, id: u8) {
    if !app.containers.values().map(|c| c.id).any(|x| x == id) {
        return;
    }
    app.state.show = Views::Remove;
    app.state.zoom_id = Some(id);
}
