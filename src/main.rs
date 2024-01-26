use crate::app::app::{run, State};
use anyhow::Result;
use std::time::Duration;

mod app;
mod taskfile;

fn main() -> Result<()> {
    let taskfile = taskfile::config::load()?;

    let mut terminal = app::terminal::setup()?;

    let tick_rate = Duration::from_millis(250);
    let state = State::new(taskfile);

    let selected_task = run(&mut terminal, state, tick_rate).unwrap();

    app::terminal::restore(&mut terminal)?;

    if let Some(task) = selected_task {
        return taskfile::command::run_task(task.name);
    } 

    Ok(())
}
