use crate::app::App;
use anyhow::Result;
use app::{
    event::{Event, EventHandler},
    terminal::UserInterface,
    update,
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod taskfile;

fn main() -> Result<()> {
    let taskfile = taskfile::config::load()?;

    let mut app = App::new(taskfile);

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = UserInterface::new(terminal, events);

    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => update(&mut app, key_event),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };
    }

    tui.exit()?;

    if let Some(task) = app.task_to_exec {
        return taskfile::command::run_task(task.name);
    }

    Ok(())
}
