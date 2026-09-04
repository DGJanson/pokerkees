// Main mod for drawing the ui
// For now we keep everything in one file
// We may expand if gets out of hand, but hopefully not

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text, ToText},
    widgets::{Block, BlockExt, Borders, Paragraph, Wrap},
};

use crate::state::AppState;

use super::layoututils::{get_inner_rect, get_inner_rect_pad, horizontal_left_divide_by_cells};

pub fn draw_ui(frame: &mut Frame, state: &mut AppState) {
    // Start with the main components
    draw_main_border(frame, state);
    let (left_box, right_box) =
        horizontal_left_divide_by_cells(get_inner_rect_pad(frame.area(), 1, 1), 50);

    draw_left_box(frame, state, left_box);
}

fn draw_main_border(frame: &mut Frame, state: &mut AppState) {
    frame.render_widget(Block::new().borders(Borders::ALL), frame.area());
    frame.render_widget(
        Paragraph::new(" PokerKees ").style(Style::default().fg(Color::Yellow)),
        Rect {
            x: 3,
            y: 0,
            width: 11,
            height: 1,
        },
    );

    let loading_state = match state.worker.worker_active {
        true => " Fetching Data ",
        false => " Fetch Inactive ",
    };

    frame.render_widget(
        Paragraph::new(loading_state).style(Style::default().fg(Color::Green)),
        Rect {
            x: frame.area().width - 18,
            y: frame.area().height - 1,
            width: 16,
            height: 1,
        },
    );
}

fn draw_left_box(frame: &mut Frame, state: &mut AppState, outer_area: Rect) {
    // Note the outer area is right inside the main border
    // For aesthetics we pad another inside, so the text looks nice
    frame.render_widget(Block::new().borders(Borders::RIGHT), outer_area);
    let area = get_inner_rect(outer_area, 1, 2, 1, 1);

    let server_health = if state.server_connected {
        Line::from("Server connected").style(Style::default().fg(Color::Green))
    } else {
        Line::from("CONNECTION FAILED").style(Style::default().fg(Color::Red))
    };

    let server_url_txt = if state.server_url.is_some() {
        state.server_url.as_ref().unwrap().as_str()
    } else {
        ""
    };

    // If we miss info in the state, the user should set these first
    if state.user_name.is_empty() || state.server_url.is_none() {
        let lines = vec![
            Line::from("Welcome to PokerKees. Please setup your options first."),
            Line::from("(Note that you can use CLI parms to set these automatically.)"),
            Line::from(""),
            Line::from("Username:").style(Style::default().fg(Color::Yellow)),
            Line::from(state.user_name.as_str()),
            Line::from("Server:").style(Style::default().fg(Color::Yellow)),
            Line::from(server_url_txt),
            Line::from(""),
            Line::from(vec![
                Span::from("Use "),
                Span::styled("N ", Style::default().fg(Color::Blue)),
                Span::from("(shift + n) to set username."),
            ]),
            Line::from(vec![
                Span::from("Use "),
                Span::styled("U ", Style::default().fg(Color::Blue)),
                Span::from("(shift + u) to set server url."),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
            area,
        );
    } else if state.selected_session.is_empty() {
        let lines = vec![
            Line::from("Username:").style(Style::default().fg(Color::Yellow)),
            Line::from(state.user_name.as_str()),
            Line::from("Server:").style(Style::default().fg(Color::Yellow)),
            Line::from(server_url_txt),
            Line::from(""),
            server_health,
            Line::from(""),
            Line::from("No session selected. You can look for one on server or create one."),
            Line::from(""),
            Line::from(vec![
                Span::from("Use "),
                Span::styled("s ", Style::default().fg(Color::Blue)),
                Span::from("to open the session menu."),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
            area,
        );
    } else {
        // Show session info
        let lines = vec![
            Line::from("Username:").style(Style::default().fg(Color::Yellow)),
            Line::from(state.user_name.as_str()),
            Line::from("Server:").style(Style::default().fg(Color::Yellow)),
            Line::from(server_url_txt),
            Line::from(""),
            server_health,
            Line::from(""),
            Line::from("Session:").style(Style::default().fg(Color::Yellow)),
            Line::from(state.selected_session.as_str()),
        ];

        frame.render_widget(
            Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
            area,
        );
    }
}
