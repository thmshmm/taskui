use crate::taskfile::config::{Task, Taskfile};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{ListItem, *},
};
use std::{
    time::{Duration, Instant},
    usize,
};

enum InputMode {
    Select,
    Search,
}

struct StatefulList {
    state: ListState,
    items: Vec<StatefulListItem>,
    orig_items: Vec<StatefulListItem>,
    last_selected: Option<usize>,
}

#[derive(Clone)]
struct StatefulListItem {
    item: Task,
}

impl StatefulList {
    fn with_items(items: Vec<Task>) -> StatefulList {
        let list_items: Vec<StatefulListItem> = items
            .into_iter()
            .map(|item| StatefulListItem { item })
            .collect();

        StatefulList {
            state: ListState::default(),
            orig_items: list_items.clone(),
            items: list_items,
            last_selected: None,
        }
    }

    fn next(&mut self) {
        if self.items.len() == 0 {
            self.reset_selected();
            return;
        }

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
        if self.items.len() == 0 {
            self.reset_selected();
            return;
        }

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

    fn reset_selected(&mut self) {
        self.state.select(None);
        self.last_selected = None;
    }

    fn get_selected(&mut self) -> Option<Task> {
        if let Some(idx) = self.state.selected() {
            Some(self.items[idx].item.clone())
        } else {
            None
        }
    }

    fn filter(&mut self, search: &String) {
        self.items = self.orig_items.clone();
        self.items.retain(|i| i.item.name.contains(search));
        self.state = ListState::default();

        if self.items.len() > 0 {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }
}

pub struct State {
    tasks: StatefulList,
    search: String,
    input_mode: InputMode,
}

impl State {
    pub fn new(taskfile: Taskfile) -> State {
        State {
            tasks: StatefulList::with_items(taskfile.tasks),
            search: String::new(),
            input_mode: InputMode::Select,
        }
    }
}

pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut state: State,
    tick_rate: Duration,
) -> Result<Option<Task>> {
    state.tasks.state.select(Some(0));

    let last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut state))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match state.input_mode {
                        InputMode::Select => match key.code {
                            KeyCode::Char('q') => return Ok(None),
                            KeyCode::Enter => {
                                if state.tasks.get_selected().is_some() {
                                    return Ok(state.tasks.get_selected());
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => state.tasks.next(),
                            KeyCode::Up | KeyCode::Char('k') => state.tasks.previous(),
                            KeyCode::Char('/') => state.input_mode = InputMode::Search,
                            _ => {}
                        },
                        InputMode::Search => match key.code {
                            KeyCode::Char(c) => {
                                state.search.push(c);
                                state.tasks.filter(&state.search);
                            }
                            KeyCode::Backspace => {
                                _ = state.search.pop();
                                state.tasks.filter(&state.search);
                            }
                            KeyCode::Esc => {
                                state.search = String::new();
                                state.tasks.filter(&state.search);
                                state.input_mode = InputMode::Select;
                            }
                            KeyCode::Enter => state.input_mode = InputMode::Select,
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut State) {
    let mut search_chunk_size = 0;

    if matches!(app.input_mode, InputMode::Search) || app.search.len() > 0 {
        search_chunk_size = 3;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(search_chunk_size), Constraint::Min(1)])
        .split(f.size());

    let items: Vec<ListItem> = app
        .tasks
        .items
        .iter()
        .map(|i| ListItem::new(i.item.name.as_str()).style(Style::default()))
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    let input = Paragraph::new(Text::from(app.search.clone()))
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Search"));

    f.render_widget(input, chunks[0]);

    match app.input_mode {
        InputMode::Search => f.set_cursor(1 + app.search.len() as u16, 1),
        _ => {}
    }

    f.render_stateful_widget(items, chunks[1], &mut app.tasks.state);
}
