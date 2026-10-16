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
    }

    // If azure config enabled
    // check for azure item info
    if state.azure_org.is_some() && state.azure_project.is_some() {
        lines.extend(show_azure_info(state));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((state.ui_state.scroll, 0)),
        inner_rect,
    );
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

fn show_azure_info(state: &AppState) -> Vec<Line> {
    let mut lines = Vec::new();
    lines.push(Line::default());
    if !state.azure_item_cache.contains_key(&state.selected_item) {
        lines.push(Line::from("No Azure info available").style(Style::default().fg(Color::Yellow)));
    } else if let Some(awi) = state.azure_item_cache.get(&state.selected_item) {
        lines.push(Line::from("Azure Devops info").style(Style::default().fg(Color::Green)));
        lines.push(Line::default());
        lines.push(Line::from("Title:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from(awi.title.as_str()));
        lines.push(Line::from("State:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from(awi.state.as_str()));
        lines.push(Line::from("Assigned to:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from(
            awi.assigned_to.as_deref().unwrap_or("Unassigned"),
        ));
        lines.push(Line::from("Tags:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from(awi.tags.join(", ")));
        lines.push(Line::from("Created:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from(awi.created_date.as_str()));
        lines.push(Line::from("Description:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from(awi.description.as_str()));
        lines.push(Line::from("Acceptance Criteria:").style(Style::default().fg(Color::Yellow)));
        lines.push(Line::from(awi.acceptance_criteria.as_str()));
    } else {
        lines.push(
            Line::from("Could not find or render Azure info")
                .style(Style::default().fg(Color::Red)),
        );
    }

    lines
}
