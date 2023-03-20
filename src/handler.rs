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
            KeyCode::Char('p') | KeyCode::Char(' ') => app.flip_pause(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::Views;
    use tui::layout::Direction;

    #[test]
    fn stop() {
        let mut app = App::default();
        app.add_container("1");
        app.add_container("2");
        assert_eq!(app.containers.len(), 2);

        // Test stoping
        app.state.running = true;
        assert_eq!(app.is_running(), true);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.is_running(), false);

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.is_running(), true);

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.is_running(), true);

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.is_running(), false);

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::CONTROL);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.is_running(), false);
    }

    #[test]
    fn flip_raw() {
        let mut app = App::default();
        app.add_container("3");
        assert_eq!(app.containers.len(), 1);
        assert_eq!(app.state.show, Views::Containers);
        let key = KeyEvent::new(KeyCode::Char('*'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show, Views::RawBuffer);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show, Views::Containers);
    }

    #[test]
    fn flip_show_input() {
        let mut app = App::default();
        assert_eq!(app.state.show_input, false);
        let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show_input, true);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show_input, false);
    }

    #[test]
    fn flip_help() {
        let mut app = App::default();
        assert_eq!(app.state.help, false);
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.help, true);
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.help, false);
    }

    #[test]
    fn flip_wrap() {
        let mut app = App::default();
        assert_eq!(app.state.wrap, false);
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.wrap, true);
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.help, false);
    }

    #[test]
    fn flip_pause() {
        let mut app = App::default();
        assert_eq!(app.state.paused, false);
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.paused, true);
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.help, false);
    }

    #[test]
    fn flip_direction() {
        let mut app = App::default();
        assert_eq!(app.state.direction, Direction::Vertical);
        let key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.direction, Direction::Horizontal);
        let key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.direction, Direction::Vertical);
    }

    #[test]
    fn scroll_up_down_continue() {
        let mut app = App::default();
        assert_eq!(app.state.scroll_up, 0);
        assert_eq!(app.state.scroll_down, 0);
        assert_eq!(app.state.paused, false);
        let mut key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 1);
        assert_eq!(app.state.scroll_down, 0);
        assert_eq!(app.state.paused, true);
        let mut key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 2);
        assert_eq!(app.state.scroll_down, 0);
        assert_eq!(app.state.paused, true);
        let mut key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 2);
        assert_eq!(app.state.scroll_down, 1);
        assert_eq!(app.state.paused, true);
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 0);
        assert_eq!(app.state.scroll_down, 0);
        assert_eq!(app.state.paused, false);
    }

    #[test]
    fn container_number() {
        let mut app = App::default();
        app.add_container("1");
        app.add_container("2");
        assert_eq!(app.containers.len(), 2);

        assert_eq!(app.state.show, Views::Containers);
        assert_eq!(app.state.zoom_id, None);
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show, Views::Zoom);
        assert_eq!(app.state.zoom_id, Some(1));
        // Flip
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show, Views::Containers);
        assert_eq!(app.state.zoom_id, None);

        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::ALT);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show, Views::Remove);
        assert_eq!(app.state.zoom_id, Some(1));
    }
}
