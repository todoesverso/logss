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
            KeyCode::Char('s') => app.flip_single_view(),
            KeyCode::Char('i') | KeyCode::Char('/') => app.flip_show_input(),
            KeyCode::Char('h') => app.flip_help(),
            KeyCode::Char('w') => app.flip_wrap(),
            KeyCode::Char('p') | KeyCode::Char(' ') => app.flip_pause(),
            KeyCode::Char('v') => app.flip_direction(),
            KeyCode::Char('1') => view_helper(app, 1, key_event),
            KeyCode::F(1) => app.hide_view(1),
            KeyCode::Char('2') => view_helper(app, 2, key_event),
            KeyCode::F(2) => app.hide_view(2),
            KeyCode::Char('3') => view_helper(app, 3, key_event),
            KeyCode::F(3) => app.hide_view(3),
            KeyCode::Char('4') => view_helper(app, 4, key_event),
            KeyCode::F(4) => app.hide_view(4),
            KeyCode::Char('5') => view_helper(app, 5, key_event),
            KeyCode::F(5) => app.hide_view(5),
            KeyCode::Char('6') => view_helper(app, 6, key_event),
            KeyCode::F(6) => app.hide_view(6),
            KeyCode::Char('7') => view_helper(app, 7, key_event),
            KeyCode::F(7) => app.hide_view(7),
            KeyCode::Char('8') => view_helper(app, 8, key_event),
            KeyCode::F(8) => app.hide_view(8),
            KeyCode::Char('9') => view_helper(app, 9, key_event),
            KeyCode::F(9) => app.hide_view(9),
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
    match key_event.modifiers {
        KeyModifiers::ALT => app.remove_view(id),
        KeyModifiers::NONE => app.zoom_into(id),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::Views;
    use ratatui::layout::Direction;
    use std::char::from_digit;

    #[test]
    fn stop() {
        let mut app = App::default();
        app.add_container("1");
        app.add_container("2");
        assert_eq!(app.containers.len(), 2);

        // Test stoping
        app.state.running = true;
        assert!(app.is_running());
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(!app.is_running());

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(app.is_running());

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(app.is_running());

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        handle_key_events(key, &mut app).ok();
        assert!(!app.is_running());

        app.state.running = true;
        let key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::CONTROL);
        handle_key_events(key, &mut app).ok();
        assert!(!app.is_running());
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
    fn flip_single() {
        let mut app = App::default();
        app.add_container("3");
        assert_eq!(app.containers.len(), 1);
        assert_eq!(app.state.show, Views::Containers);
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show, Views::SingleBuffer);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.show, Views::Containers);
    }

    #[test]
    fn flip_show_input() {
        let mut app = App::default();
        assert!(!app.state.show_input);
        let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(app.state.show_input);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(!app.state.show_input);
    }

    #[test]
    fn flip_help() {
        let mut app = App::default();
        assert!(!app.state.help);
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(app.state.help);
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(!app.state.help);
    }

    #[test]
    fn flip_wrap() {
        let mut app = App::default();
        assert!(!app.state.wrap);
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(app.state.wrap);
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(!app.state.help);
    }

    #[test]
    fn flip_pause() {
        let mut app = App::default();
        assert!(!app.state.paused);
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(app.state.paused);
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert!(!app.state.help);
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
        assert!(!app.state.paused);
        let mut key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 1);
        assert_eq!(app.state.scroll_down, 0);
        assert!(app.state.paused);
        let mut key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 2);
        assert_eq!(app.state.scroll_down, 0);
        assert!(app.state.paused);
        let mut key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        key.kind = KeyEventKind::Press;
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 2);
        assert_eq!(app.state.scroll_down, 1);
        assert!(app.state.paused);
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        handle_key_events(key, &mut app).ok();
        assert_eq!(app.state.scroll_up, 0);
        assert_eq!(app.state.scroll_down, 0);
        assert!(!app.state.paused);
    }

    #[test]
    fn container_number() {
        for i in 1..9_u8 {
            let mut app = App::default();
            for a in 1..9_u8 {
                app.add_container(&a.to_string());
            }
            assert_eq!(app.state.show, Views::Containers);
            assert_eq!(app.state.zoom_id, None);
            let key = KeyEvent::new(
                KeyCode::Char(from_digit(i as u32, 10).unwrap()),
                KeyModifiers::NONE,
            );
            handle_key_events(key, &mut app).ok();
            assert_eq!(app.state.show, Views::Zoom);
            assert_eq!(app.state.zoom_id, Some(i));
            // Flip
            handle_key_events(key, &mut app).ok();
            assert_eq!(app.state.show, Views::Containers);
            assert_eq!(app.state.zoom_id, None);
            // remove
            let key = KeyEvent::new(
                KeyCode::Char(from_digit(i as u32, 10).unwrap()),
                KeyModifiers::ALT,
            );
            handle_key_events(key, &mut app).ok();
            assert_eq!(app.state.show, Views::Remove);
            assert_eq!(app.state.zoom_id, Some(i));
        }
    }

    #[test]
    fn container_hide() {
        let mut app = App::default();
        for i in 1..9_u8 {
            app.add_container(&i.to_string());
            assert!(!app.containers[(i - 1) as usize].state.hide);
            let key = KeyEvent::new(KeyCode::F(i), KeyModifiers::NONE);
            handle_key_events(key, &mut app).ok();
            assert!(app.containers[(i - 1) as usize].state.hide);
        }
    }
}
