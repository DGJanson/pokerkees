use poker_shared::PokerItem;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::{state::AppState, tui::layoututils::get_inner_rect};

pub fn draw_item_screen(frame: &mut Frame, state: &mut AppState, area: Rect) {
    let inner_rect = get_inner_rect(area, 2, 2, 1, 1);

    let mut lines = get_header_lines(state);

    if let Some(item_info) = state.selected_item_info.as_ref() {
        lines.extend(show_item_info(item_info, state));
    } else {
        lines.push(Line::from("No Item Info available yet."));
        lines.push(Line::default());
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner_rect);
}

fn get_header_lines(state: &AppState) -> Vec<Line> {
    let lines = vec![
        Line::from(vec![
            Span::from("Use "),
            Span::from("v").style(Style::default().fg(Color::Blue)),
            Span::from(" to cast your vote. Use "),
            Span::from("R").style(Style::default().fg(Color::Blue)),
            Span::from(" to reveal votes for everyone. Use "),
            Span::from("C").style(Style::default().fg(Color::Blue)),
            Span::from(" to close this item for everyone."),
        ]),
        Line::default(),
        Line::from(vec![
            Span::from("ITEM ").style(Style::default().fg(Color::Yellow)),
            Span::from(format!("{}", state.selected_item))
                .style(Style::default().fg(Color::Magenta)),
        ]),
        Line::default(),
    ];

    lines
}

fn show_item_info<'a>(item: &'a PokerItem, state: &'a AppState) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    if item.done {
        lines.push(Line::from("THIS ITEM IS CLOSED.").style(Style::default().fg(Color::Red)));
        lines.push(Line::default());
    }

    lines.push(Line::from(item.item_title.as_str()).style(Style::default().fg(Color::Yellow)));
    lines.push(Line::default());

    // Did current user vote
    let voted = item.votes.contains_key(state.user_name.as_str());
    let nr_votes = item.votes.len();

    if !item.revealed {
        lines.push(Line::from(vec![
            Span::from("There are "),
            Span::from(format!("{nr_votes}")).style(Style::default().fg(Color::Magenta)),
            Span::from(" votes on this item"),
        ]));
        lines.push(Line::default());
        if voted {
            let vote_value = item
                .votes
                .get(state.user_name.as_str())
                .copied()
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::from("You have voted for this item -> "),
                Span::from(format!("{vote_value}")).style(Style::default().fg(Color::Magenta)),
            ]))
        } else {
            lines.push(Line::from("You have not voted yet."));
        }
        lines.push(Line::default());
    } else {
        lines.push(Line::from("Votes:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::default());
        // We use the known users to show votes
        // start with own vote
        if let Some(v) = item.votes.get(&state.user_name) {
            lines.push(
                Line::from(format!("{:<20} {v:2}", &state.user_name))
                    .style(Style::default().fg(Color::Magenta)),
            );
        } else {
            lines.push(
                Line::from(format!("{:<20}  -", &state.user_name))
                    .style(Style::default().fg(Color::Magenta)),
            );
        }

        for name in state.known_users.iter() {
            if let Some(v) = item.votes.get(name) {
                lines.push(Line::from(format!("{name:<20} {v:2}")));
            } else {
                lines.push(Line::from(format!("{name:<20}  -")));
            }
        }
    }

    lines
}
