use crate::animation::{ActivePane, AnimationEngine};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct EditorPane;

impl EditorPane {
    fn highlight_line(
        &self,
        line_content: &str,
        line_num: usize,
        show_cursor: bool,
        cursor_col: usize,
        highlights: &[crate::syntax::HighlightSpan],
        buffer_lines: &[String],
        editing_line: Option<usize>,
        editing_insert_byte_position: usize,
        editing_line_byte_offset: isize,
    ) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Calculate byte offset for this line
        let mut byte_offset = 0;
        for (i, line) in buffer_lines.iter().enumerate() {
            if i == line_num {
                break;
            }
            byte_offset += line.len() + 1; // +1 for newline
        }

        // Calculate editing line's byte offset if needed
        let editing_line_byte_offset_abs = if let Some(edit_line) = editing_line {
            if edit_line < line_num {
                // Current line is after the editing line - need to shift by offset
                Some(editing_line_byte_offset)
            } else if edit_line == line_num {
                // This is the editing line - will handle specially
                None
            } else {
                // Current line is before editing line - no offset needed
                None
            }
        } else {
            None
        };

        // Filter highlights for this line and adjust their ranges
        let line_end = byte_offset + line_content.len();
        let line_highlights: Vec<_> = highlights.iter()
            .filter_map(|h| {
                let adjusted_start;
                let adjusted_end;

                if editing_line == Some(line_num) {
                    // This is the line being edited - adjust highlights within this line
                    let insert_abs_pos = byte_offset + editing_insert_byte_position;

                    // Adjust start position
                    if h.start >= insert_abs_pos {
                        // Highlight is after insertion point - shift it
                        adjusted_start = (h.start as isize + editing_line_byte_offset).max(0) as usize;
                    } else {
                        adjusted_start = h.start;
                    }

                    // Adjust end position
                    if h.end > insert_abs_pos {
                        // Highlight extends past insertion point - shift end
                        adjusted_end = (h.end as isize + editing_line_byte_offset).max(0) as usize;
                    } else {
                        adjusted_end = h.end;
                    }
                } else if let Some(offset) = editing_line_byte_offset_abs {
                    // Line is after editing line - shift entire highlight
                    adjusted_start = (h.start as isize + offset).max(0) as usize;
                    adjusted_end = (h.end as isize + offset).max(0) as usize;
                } else {
                    // No adjustment needed
                    adjusted_start = h.start;
                    adjusted_end = h.end;
                }

                // Filter to this line
                if adjusted_start < line_end && adjusted_end > byte_offset {
                    Some((adjusted_start, adjusted_end, h.token_type))
                } else {
                    None
                }
            })
            .collect();

        if show_cursor {
            // Apply syntax highlighting with cursor
            let chars: Vec<char> = line_content.chars().collect();

            for (char_idx, ch) in chars.iter().enumerate() {
                let char_byte_start = byte_offset + line_content.chars().take(char_idx).map(|c| c.len_utf8()).sum::<usize>();
                let char_byte_end = char_byte_start + ch.len_utf8();

                // Find highlight for this character
                let color = line_highlights.iter()
                    .find(|h| char_byte_start >= h.0 && char_byte_end <= h.1)
                    .map(|h| h.2.color())
                    .unwrap_or(Color::White);

                if char_idx == cursor_col {
                    // Cursor character
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default()
                            .bg(Color::White)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                }
            }

            // Add cursor if at end of line
            if cursor_col >= chars.len() {
                spans.push(Span::styled(
                    " ",
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        } else {
            // Apply syntax highlighting without cursor
            let chars: Vec<char> = line_content.chars().collect();

            for (char_idx, ch) in chars.iter().enumerate() {
                let char_byte_start = byte_offset + line_content.chars().take(char_idx).map(|c| c.len_utf8()).sum::<usize>();
                let char_byte_end = char_byte_start + ch.len_utf8();

                // Find highlight for this character
                let color = line_highlights.iter()
                    .find(|h| char_byte_start >= h.0 && char_byte_end <= h.1)
                    .map(|h| h.2.color())
                    .unwrap_or(Color::White);

                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
        }

        spans
    }

    pub fn render(&self, f: &mut Frame, area: Rect, engine: &AnimationEngine) {
        let block = Block::default()
            .title("Editor")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        // Get visible lines based on scroll offset and area height
        let content_height = area.height.saturating_sub(2) as usize; // Subtract borders
        let scroll_offset = engine.buffer.scroll_offset;
        let buffer_lines = &engine.buffer.lines;

        // Calculate line number width
        let total_lines = buffer_lines.len();
        let line_num_width = format!("{}", total_lines).len().max(3);

        // Use cached syntax highlights from buffer
        let highlights = &engine.buffer.cached_highlights;

        let visible_lines: Vec<Line> = buffer_lines
            .iter()
            .skip(scroll_offset)
            .take(content_height)
            .enumerate()
            .map(|(idx, line_content)| {
                let line_num = scroll_offset + idx;
                let is_cursor_line = line_num == engine.buffer.cursor_line;

                let mut spans = Vec::new();

                // Line number
                let line_num_str = format!("{:>width$} ", line_num + 1, width = line_num_width);
                if is_cursor_line {
                    spans.push(Span::styled(
                        line_num_str,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        line_num_str,
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                // Line separator
                spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));

                // Get syntax highlighting for this line
                let line_spans = self.highlight_line(
                    line_content,
                    line_num,
                    is_cursor_line && engine.cursor_visible && engine.active_pane == ActivePane::Editor,
                    engine.buffer.cursor_col,
                    &highlights,
                    buffer_lines,
                    engine.buffer.editing_line,
                    engine.buffer.editing_insert_byte_position,
                    engine.buffer.editing_line_byte_offset,
                );
                spans.extend(line_spans);

                Line::from(spans)
            })
            .collect();

        let content = Paragraph::new(visible_lines).block(block);
        f.render_widget(content, area);
    }
}
