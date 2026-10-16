// File that draws the session part of the screen

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{state::AppState, tui::layoututils::get_inner_rect_pad};

pub fn draw_session_info(frame: &mut Frame, state: &mut AppState, area: Rect) {
    let inner_rect = get_inner_rect_pad(area, 2, 1);

    let mut content = get_session_header();
    content.extend(get_available_sessions(state));

    // Show delete instruction in case of session selected
    if !state.selected_session.is_empty() {
        content.extend(get_delete_instruction());
    }

    frame.render_widget(Paragraph::new(content), inner_rect);
}

fn get_session_header() -> Vec<Line<'static>> {
    let lines = vec![
        Line::from("SESSIONS").style(Style::default().fg(Color::Yellow)),
        Line::default(),
        Line::from(vec![
            Span::from(
                "This screen shows the available sessions. If you want to create a new one, press: ",
            ),
            Span::from("C").style(Style::default().fg(Color::Blue)),
        ]),
    ];

    lines
}

fn get_available_sessions(state: &AppState) -> Vec<Line<'_>> {
    let mut lines = vec![
        Line::default(),
        Line::from("Available sessions:").style(Style::default().fg(Color::Magenta)),
        Line::default(),
    ];
    if state.available_sessions.is_empty() {
        lines.push(Line::from("No sessions found on server."));
    } else {
        for (i, s) in state.available_sessions.iter().enumerate() {
            // We use the selected index to select available sessions
            if i == state.ui_state.selected_index {
                lines.push(
                    Line::from(s.as_str()).style(Style::default().fg(Color::White).bg(Color::Blue)),
                );
            } else {
                lines.push(Line::from(s.as_str()));
            }
        }
        // Som extra info
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::from("Use "),
            Span::from("j").style(Style::default().fg(Color::Blue)),
            Span::from(" and "),
            Span::from("k").style(Style::default().fg(Color::Blue)),
            Span::from(" keys to select available sessions. Press "),
            Span::from("enter").style(Style::default().fg(Color::Blue)),
            Span::from(" key to confirm selection of session."),
        ]));
        lines.push(Line::default());
    }

    lines
}

fn get_delete_instruction() -> Vec<Line<'static>> {
    let lines = vec![
        Line::default(),
        Line::from(vec![
            Span::from("To delete the current active session press "),
            Span::from("D").style(Style::default().fg(Color::Blue)),
            Span::from(". But note that it will delete the session for everyone"),
        ]),
    ];

    lines
}
