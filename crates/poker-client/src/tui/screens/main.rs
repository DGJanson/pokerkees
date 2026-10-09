// Code for drawing the main portion of the screen.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{state::AppState, tui::layoututils::get_inner_rect};

/// Draws the main screen for user interaction with the poker session
/// we assume this function gets called when there is an active session
/// otherwise there is not that much point in calling this function
pub fn draw_main_screen(frame: &mut Frame, state: &mut AppState, area: Rect) {
    let inner_rect = get_inner_rect(area, 2, 2, 1, 1);

    let mut lines = Vec::<Line>::new();

    lines.extend(get_header_line());

    // if there are availabe items show them here!
    if !state.selected_session.is_empty() {
        lines.extend(show_items_info(state));
    }

    frame.render_widget(Paragraph::new(lines), inner_rect);
}

/// Create the first lines,
/// these are always the same
/// regardless of state
fn get_header_line() -> Vec<Line<'static>> {
    let lines = vec![
        Line::from("POKER").style(Style::default().fg(Color::Yellow)),
        Line::default(),
        Line::from(vec![
            Span::from(
                "Interact with the poker session. You can select items below. Create a new item by pressing ",
            ),
            Span::from("C").style(Style::default().fg(Color::Blue)),
            Span::from("."),
        ]),
        Line::default(),
    ];

    lines
}

/// Show available items
fn show_items_info(state: &AppState) -> Vec<Line> {
    let mut lines = Vec::new();
    if state.active_items.is_empty() {
        // Base case, no available items yet
        lines.push(Line::from("This sessions does not have items yet"));
        lines.push(Line::default());
    } else {
        // header
        lines.push(Line::from("Active Items").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from("Item Nr   Revealed").style(Style::default().fg(Color::Yellow)));
        // simply iterate over active items
        for (index, item) in state.active_items.iter().enumerate() {
            if index == state.ui_state.selected_index {
                lines.push(
                    Line::from(format!(
                        "{:<10}{:<10}{}",
                        item.id, item.revealed, item.item_title
                    ))
                    .style(Style::default().bg(Color::LightBlue)),
                );
            } else {
                lines.push(Line::from(format!(
                    "{:<10}{:<10}{}",
                    item.id, item.revealed, item.item_title
                )));
            }
        }

        lines.push(Line::default());
    }
    // Show number of closed items
    lines.push(Line::from(vec![
        Span::from("There are "),
        Span::from(format!("{}", state.closed_items.len()))
            .style(Style::default().fg(Color::Magenta)),
        Span::from(" closed items."), // TODO, add function to reveal closed items?
    ]));

    lines
}
