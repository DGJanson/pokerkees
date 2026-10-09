// Main mod for drawing the ui
// For now we keep everything in one file
// We may expand if gets out of hand, but hopefully not

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::state::{AppState, UIScreenMode};

use super::{
    layoututils::{
        get_center_rect, get_inner_rect, get_inner_rect_pad, horizontal_left_divide_by_cells,
    },
    screens::session::draw_session_info,
    screens::main::draw_main_screen,
};

pub fn draw_ui(frame: &mut Frame, state: &mut AppState) {
    // Start with the main components
    draw_main_border(frame, state);
    let (left_box, right_box) =
        horizontal_left_divide_by_cells(get_inner_rect_pad(frame.area(), 1, 1), 50);

    draw_left_box(frame, state, left_box);
    draw_right_box(frame, state, right_box);

    // If there is an error, we ALWAYS draw it at top of rightbox
    if state.app_error.is_some() {
        draw_error(frame, state.app_error.as_deref(), right_box);
        return;
    }

    if state.worker_error.is_some() {
        draw_error(frame, state.worker_error.as_deref(), right_box);
    }
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

// !!! ----- !!!
// The error is simple:
// A big red string at top of right box.
fn draw_error(frame: &mut Frame, error: Option<&str>, outer_area: Rect) {
    let error_box = get_inner_rect(outer_area, 2, 2, 1, outer_area.height - 7);
    let error_msx_box = get_inner_rect_pad(error_box, 2, 2);

    frame.render_widget(Clear, error_box);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
        error_box,
    );
    let error_msg = error.unwrap_or_default();
    frame.render_widget(
        Paragraph::new(error_msg)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Red)),
        error_msx_box,
    );
}

// !!! ----- !!!
// The RIGHT BOX contains info on the selected unit (item, session, etc.)
// It is probably the most interactive and hence earlier in the code :)

fn draw_right_box(frame: &mut Frame, state: &mut AppState, outer_area: Rect) {
    // What to draw exactly depends a lot on the state, hence we
    // match on state right away
    match state.ui_state.screen_mode {
        UIScreenMode::Main => {
            draw_main_screen(frame, state, outer_area);
        }
        UIScreenMode::Session => {
            draw_session_info(frame, state, outer_area);
        }
        UIScreenMode::Input => {
            draw_input_box(frame, state, outer_area);
        }
    }
}

fn draw_input_box(frame: &mut Frame, state: &mut AppState, outer_area: Rect) {
    // only draw something in the center of the screen
    let border_box = get_center_rect(outer_area, outer_area.width - 4, 7);
    frame.render_widget(Block::bordered(), border_box);
    let content_box = get_inner_rect_pad(border_box, 2, 2);
    // First we draw the input type
    // Then the value
    let lines = vec![
        Line::from(state.ui_state.input_type.to_string()).style(Style::default().fg(Color::Yellow)),
        Line::from(state.ui_state.input_value.as_str()),
    ];
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        content_box,
    );
}

// !!! ----- !!!
// The LEFT BOX contains general info on app state

fn draw_left_box(frame: &mut Frame, state: &mut AppState, outer_area: Rect) {
    // Note the outer area is right inside the main border
    // For aesthetics we pad another inside, so the text looks nice
    frame.render_widget(Block::new().borders(Borders::RIGHT), outer_area);
    let area = get_inner_rect(outer_area, 1, 2, 1, 1);

    let mut lines = if state.user_name.is_empty() || state.server_url.is_none() {
        build_setup_lines(state)
    } else if state.selected_session.is_empty() {
        build_no_session_lines(state)
    } else {
        build_session_lines(state)
    };

    lines.extend(show_available_shortcuts_left(state));

    frame.render_widget(
        Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
        area,
    );
}

fn server_health_line(state: &AppState) -> Line<'static> {
    if state.server_connected {
        Line::from("Server connected").style(Style::default().fg(Color::Green))
    } else {
        Line::from("CONNECTION FAILED").style(Style::default().fg(Color::Red))
    }
}

fn server_url_text(state: &AppState) -> &str {
    if let Some(url) = state.server_url.as_ref() {
        url.as_str()
    } else {
        ""
    }
}

fn header_lines(state: &AppState, include_health: bool) -> Vec<Line> {
    let mut lines = vec![
        Line::from("Username:").style(Style::default().fg(Color::Yellow)),
        Line::from(state.user_name.as_str()),
        Line::from("Server:").style(Style::default().fg(Color::Yellow)),
        Line::from(server_url_text(state)),
    ];

    if include_health {
        lines.push(Line::from(""));
        lines.push(server_health_line(state));
        lines.push(Line::from(""));
    }

    lines
}

fn build_setup_lines(state: &AppState) -> Vec<Line> {
    let mut lines = vec![
        Line::from("Welcome to PokerKees. Please setup your options first."),
        Line::from("(Note that you can use CLI parms to set these automatically.)"),
        Line::from(""),
    ];
    lines.extend(header_lines(state, false));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::from("Use "),
        Span::styled("N ", Style::default().fg(Color::Blue)),
        Span::from("(shift + n) to set username."),
    ]));
    lines.push(Line::from(vec![
        Span::from("Use "),
        Span::styled("U ", Style::default().fg(Color::Blue)),
        Span::from("(shift + u) to set server url."),
    ]));
    lines
}

fn build_no_session_lines(state: &AppState) -> Vec<Line> {
    let mut lines = header_lines(state, true);
    lines.push(Line::from(
        "No session selected. You can look for one on server or create one.",
    ));
    lines.push(Line::from(""));
    lines
}

fn build_session_lines(state: &AppState) -> Vec<Line> {
    let mut lines = header_lines(state, true);
    lines.push(Line::from("Session:").style(Style::default().fg(Color::Yellow)));
    lines.push(Line::from(state.selected_session.as_str()));
    lines.push(Line::from(""));
    lines
}

fn show_available_shortcuts_left(state: &AppState) -> Vec<Line> {
    let mut lines = Vec::new();
    if state.server_connected && state.ui_state.screen_mode == UIScreenMode::Main {
        lines.push(Line::from(vec![
            Span::from("Use "),
            Span::styled("s ", Style::default().fg(Color::Blue)),
            Span::from("to open the session menu."),
        ]));
    }

    lines
}
