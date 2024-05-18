use anyhow::Result;
use crossterm::terminal;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stderr;
use std::{io, panic};

use super::event::EventHandler;
use super::{app, ui};

pub type CrosstermTerminal = Terminal<CrosstermBackend<Stderr>>;

pub struct UserInterface {
    terminal: CrosstermTerminal,
    pub events: EventHandler,
}

impl UserInterface {
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn draw(&mut self, state: &mut app::App) -> Result<()> {
        self.terminal.draw(|frame| ui::render(frame, state))?;

        Ok(())
    }

    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;

        self.terminal.show_cursor()?;

        Ok(())
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;

        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }
}
