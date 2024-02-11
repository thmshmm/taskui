use ratatui::{
    prelude::*,
    widgets::{ListItem, *},
};

use super::app::{App, InputMode};

pub fn render(f: &mut Frame, app: &mut App) {
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
