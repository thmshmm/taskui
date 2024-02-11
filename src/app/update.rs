use crossterm::event::{KeyCode, KeyEvent};

use super::app::{App, InputMode};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match app.input_mode {
        InputMode::Select => match key_event.code {
            KeyCode::Char('q') => app.quit(),
            KeyCode::Enter => {
                if app.tasks.get_selected().is_some() {
                    app.task_to_exec = app.tasks.get_selected();
                    app.quit()
                }
            }
            KeyCode::Down | KeyCode::Char('j') => app.tasks.next(),
            KeyCode::Up | KeyCode::Char('k') => app.tasks.previous(),
            KeyCode::Char('/') => app.input_mode = InputMode::Search,
            _ => {}
        },
        InputMode::Search => match key_event.code {
            KeyCode::Char(c) => {
                app.search.push(c);
                app.tasks.filter(&app.search);
            }
            KeyCode::Backspace => {
                _ = app.search.pop();
                app.tasks.filter(&app.search);
            }
            KeyCode::Esc => {
                app.search = String::new();
                app.tasks.filter(&app.search);
                app.input_mode = InputMode::Select;
            }
            KeyCode::Enter => app.input_mode = InputMode::Select,
            _ => {}
        },
    }
}
