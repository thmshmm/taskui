use std::str::FromStr;

use ratatui::{
    prelude::*,
    widgets::{ListItem, *},
};

use super::app::{App, InputMode};

pub fn render(f: &mut Frame, app: &mut App) {
    let mut search_chunk_size = 0;

    if matches!(app.input_mode, InputMode::Search) || !app.search.is_empty() {
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
                .fg(Color::from_str("#0000ff").unwrap())
                .add_modifier(Modifier::BOLD),
        );

    let input = Paragraph::new(Text::from(app.search.clone()))
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Search"));

    f.render_widget(input, chunks[0]);

    f.render_stateful_widget(items, chunks[1], &mut app.tasks.state);

    match app.input_mode {
        InputMode::Search => f.set_cursor(1 + app.search.len() as u16, 1),
        InputMode::Preview => render_preview(f, app),
        _ => {}
    }
}

pub fn render_preview(f: &mut Frame, app: &mut App) {
    let selected_task = app.tasks.get_selected().unwrap();
    let area = centered_rect(70, 90, f.size());
    let paragraph = Paragraph::new(selected_task.body.as_str())
        .alignment(Alignment::Left)
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Preview: {}",
            selected_task.name.split(':').last().unwrap()
        )));

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
