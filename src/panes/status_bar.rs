use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding},
    Frame,
};

use crate::git::CommitMetadata;
use crate::theme::Theme;
use crate::widgets::SelectableParagraph;

pub struct StatusBarPane;

impl StatusBarPane {
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        metadata: Option<&CommitMetadata>,
        theme: &Theme,
    ) {
        let block = Block::default()
            .style(Style::default().bg(theme.background_left))
            .padding(Padding::vertical(1));

        let status_text = if let Some(meta) = metadata {
            let is_working_tree = meta.hash == "working-tree";
            let hash_display = if is_working_tree {
                "working"
            } else {
                &meta.hash[..7.min(meta.hash.len())]
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::raw("hash: "),
                    Span::styled(hash_display, Style::default().fg(theme.status_hash)),
                ]),
                Line::from(vec![
                    Span::raw("author: "),
                    Span::styled(&meta.author, Style::default().fg(theme.status_author)),
                ]),
            ];

            // Only show date for actual commits (not working tree)
            if !is_working_tree {
                let date_str = meta.date.format("%Y-%m-%d %H:%M:%S").to_string();
                lines.push(Line::from(vec![
                    Span::raw("date: "),
                    Span::styled(date_str, Style::default().fg(theme.status_date)),
                ]));
            }

            // Add commit message lines (skip empty lines)
            for msg_line in meta.message.lines() {
                if !msg_line.trim().is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        msg_line,
                        Style::default().fg(theme.status_message),
                    )]));
                }
            }

            lines
        } else {
            vec![Line::from(vec![Span::styled(
                "No commit loaded",
                Style::default().fg(theme.status_no_commit),
            )])]
        };

        let content = SelectableParagraph::new(status_text)
            .block(block)
            .background_style(Style::default().bg(theme.background_left))
            .padding(Padding::horizontal(2));

        f.render_widget(content, area);
    }
}
