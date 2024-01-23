use crate::taskfile::config::Taskfile;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::{
    time::{Duration, Instant},
    usize,
};

struct StatefulList {
    state: ListState,
    items: Vec<String>,
    last_selected: Option<usize>,
}

impl StatefulList {
    fn with_items(items: Vec<String>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
            last_selected: None,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };

        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };

        self.state.select(Some(i));
    }

    fn get_selected(&mut self) -> String {
        let idx = self.state.selected().unwrap();
        self.items[idx].clone()
    }
}

pub struct State {
    tasks: StatefulList,
}

impl State {
    pub fn new(taskfile: Taskfile) -> State {
        State {
            tasks: StatefulList::with_items(taskfile.tasks),
        }
    }
}

pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut state: State,
    tick_rate: Duration,
) -> Result<String> {
    state.tasks.state.select(Some(0));

    let last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut state))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok("".to_string()),
                        KeyCode::Enter => return Ok(state.tasks.get_selected()),
                        KeyCode::Down | KeyCode::Char('j') => state.tasks.next(),
                        KeyCode::Up | KeyCode::Char('k') => state.tasks.previous(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut State) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(f.size());

    let items: Vec<ListItem> = app
        .tasks
        .items
        .iter()
        .map(|i| ListItem::new(i.as_str()).style(Style::default()))
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(items, chunks[0], &mut app.tasks.state);
}
