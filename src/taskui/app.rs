use crate::taskfile::config::Task;
use ratatui::widgets::ListState;
use std::usize;

use super::Config;

pub struct App {
    pub cfg: Config,
    pub tasks: StatefulList,
    pub search: String,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub task_to_exec: Option<Task>,
}

impl App {
    pub fn new(cfg: Config, tasks: Vec<Task>) -> App {
        let tasks = tasks
            .into_iter()
            .filter(|task| !task.internal || cfg.list_internal)
            .collect();

        App {
            cfg,
            tasks: StatefulList::with_items(tasks),
            search: String::new(),
            input_mode: InputMode::Select,
            should_quit: false,
            task_to_exec: None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

pub enum InputMode {
    Select,
    Search,
    Preview,
}

pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<StatefulListItem>,
    orig_items: Vec<StatefulListItem>,
    last_selected: Option<usize>,
}

#[derive(Clone)]
pub struct StatefulListItem {
    pub item: Task,
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

    pub fn next(&mut self) {
        if self.items.is_empty() {
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

    pub fn previous(&mut self) {
        if self.items.is_empty() {
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

    pub fn get_selected(&mut self) -> Option<Task> {
        if let Some(idx) = self.state.selected() {
            Some(self.items[idx].item.clone())
        } else {
            None
        }
    }

    pub fn filter(&mut self, search: &String) {
        self.items.clone_from(&self.orig_items);
        self.items.retain(|i| i.item.name.contains(search));
        self.state = ListState::default();

        if !self.items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }
}
