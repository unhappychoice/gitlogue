use crate::git::{CommitMetadata, LineChangeType};
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};
use std::collections::BTreeMap;
use unicode_width::UnicodeWidthStr;

type FileEntry = (usize, String, String, Color, usize, usize);
type FileTree = BTreeMap<String, Vec<FileEntry>>;

pub struct FileTreePane {
    cached_lines: Vec<Line<'static>>,
    cached_current_line_index: Option<usize>,
    cached_total_display_lines: usize,
    cached_metadata_id: Option<String>,
    cached_current_file_index: Option<usize>,
    cached_content_width: Option<usize>,
}

impl FileTreePane {
    pub fn new() -> Self {
        Self {
            cached_lines: vec![Line::from("No commit loaded")],
            cached_current_line_index: None,
            cached_total_display_lines: 1,
            cached_metadata_id: None,
            cached_current_file_index: None,
            cached_content_width: None,
        }
    }

    pub fn set_commit_metadata(
        &mut self,
        metadata: &CommitMetadata,
        current_file_index: usize,
        theme: &Theme,
        content_width: usize,
    ) {
        let metadata_id = metadata.hash.clone();

        // Only recalculate if metadata, current file, or content width changed
        if self.cached_metadata_id.as_ref() == Some(&metadata_id)
            && self.cached_current_file_index == Some(current_file_index)
            && self.cached_content_width == Some(content_width)
        {
            return;
        }

        let (lines, current_line_index, total_display_lines) =
            Self::build_tree_lines(metadata, current_file_index, theme, content_width);

        self.cached_lines = lines;
        self.cached_current_line_index = current_line_index;
        self.cached_total_display_lines = total_display_lines;
        self.cached_metadata_id = Some(metadata_id);
        self.cached_current_file_index = Some(current_file_index);
        self.cached_content_width = Some(content_width);
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .style(Style::default().bg(theme.background_left))
            .padding(Padding {
                left: 0,
                right: 0,
                top: 1,
                bottom: 1,
            });

        // Calculate scroll offset to keep current file visible
        // Subtract padding (top: 1, bottom: 1)
        let visible_height = area.height.saturating_sub(2) as usize;
        let scroll_offset = if let Some(line_idx) = self.cached_current_line_index {
            Self::calculate_scroll_offset(line_idx, visible_height, self.cached_total_display_lines)
        } else {
            0
        };

        let content = Paragraph::new(self.cached_lines.clone())
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset as u16, 0));
        f.render_widget(content, area);
    }

    fn calculate_scroll_offset(
        current_line: usize,
        visible_height: usize,
        total_lines: usize,
    ) -> usize {
        if visible_height == 0 || total_lines == 0 {
            return 0;
        }

        // If all lines fit in viewport, no need to scroll
        if total_lines <= visible_height {
            return 0;
        }

        // Keep the current line in the middle of the viewport
        let preferred_position = visible_height / 2;

        let offset = current_line.saturating_sub(preferred_position);

        // Make sure we don't scroll past the end
        let max_offset = total_lines.saturating_sub(visible_height);
        offset.min(max_offset)
    }

    fn calculate_wrapped_lines(text_display_width: usize, area_width: usize) -> usize {
        if area_width == 0 {
            return 1;
        }
        text_display_width.div_ceil(area_width).max(1)
    }

    fn build_tree_lines(
        metadata: &CommitMetadata,
        current_file_index: usize,
        theme: &Theme,
        area_width: usize,
    ) -> (Vec<Line<'static>>, Option<usize>, usize) {
        // Build directory tree
        let mut tree: FileTree = BTreeMap::new();

        for (index, change) in metadata.changes.iter().enumerate() {
            let (status_char, color) = match change.status.as_str() {
                "A" => ("+", theme.file_tree_added),
                "D" => ("-", theme.file_tree_deleted),
                "M" => ("~", theme.file_tree_modified),
                "R" => (">", theme.file_tree_renamed),
                _ => (" ", theme.file_tree_default),
            };

            // Count additions and deletions
            let mut additions = 0;
            let mut deletions = 0;
            for hunk in &change.hunks {
                for line in &hunk.lines {
                    match line.change_type {
                        LineChangeType::Addition => additions += 1,
                        LineChangeType::Deletion => deletions += 1,
                        _ => {}
                    }
                }
            }

            let parts: Vec<&str> = change.path.split('/').collect();
            if parts.len() == 1 {
                // Root level file
                tree.entry("".to_string()).or_default().push((
                    index,
                    change.path.clone(),
                    status_char.to_string(),
                    color,
                    additions,
                    deletions,
                ));
            } else {
                // File in directory
                let dir = parts[..parts.len() - 1].join("/");
                let filename = parts[parts.len() - 1].to_string();
                tree.entry(dir).or_default().push((
                    index,
                    filename,
                    status_char.to_string(),
                    color,
                    additions,
                    deletions,
                ));
            }
        }

        let mut lines = Vec::new();
        let mut current_line_index = None;
        let mut display_line_count = 0; // Track actual display lines including wrapping
        let sorted_dirs: Vec<_> = tree.keys().cloned().collect();

        for dir in sorted_dirs {
            let mut files = tree.get(&dir).unwrap().clone();
            // Sort files by filename within each directory
            files.sort_by(|a, b| a.1.cmp(&b.1));

            // Add directory header if not root
            if !dir.is_empty() {
                let dir_text = format!("{}/", dir);
                let dir_content_width = 2 + dir_text.width(); // left padding + text

                let mut dir_spans = vec![
                    Span::styled(
                        "  ", // left padding
                        Style::default().bg(theme.background_left),
                    ),
                    Span::styled(
                        dir_text.clone(),
                        Style::default()
                            .fg(theme.file_tree_directory)
                            .bg(theme.background_left)
                            .add_modifier(Modifier::BOLD),
                    ),
                ];

                // Fill to right edge with background color
                if dir_content_width < area_width {
                    dir_spans.push(Span::styled(
                        " ".repeat(area_width - dir_content_width),
                        Style::default().bg(theme.background_left),
                    ));
                }

                lines.push(Line::from(dir_spans));
                display_line_count += Self::calculate_wrapped_lines(dir_content_width, area_width);
            }

            // Add files
            for (index, filename, status_char, color, additions, deletions) in &files {
                let is_current = *index == current_file_index;

                // Track the line index of the current file (before adding the line)
                if is_current {
                    current_line_index = Some(display_line_count);
                }

                let indent = if dir.is_empty() { "" } else { "  " }.to_string();
                let status_str = format!("{} ", status_char);
                let stats_str = format!(" +{} -{}", additions, deletions);

                // Calculate actual line width for wrapping (use display width for wide characters)
                // Include left padding (2) in the width calculation
                let content_width =
                    2 + indent.width() + status_str.width() + filename.width() + stats_str.width();

                let mut spans = vec![];

                if is_current {
                    // Current file - apply background to entire line
                    // Add left padding
                    spans.push(Span::styled(
                        "  ",
                        Style::default().bg(theme.file_tree_current_file_bg),
                    ));
                    spans.push(Span::styled(
                        indent.clone(),
                        Style::default().bg(theme.file_tree_current_file_bg),
                    ));
                    spans.push(Span::styled(
                        status_str.clone(),
                        Style::default()
                            .fg(*color)
                            .bg(theme.file_tree_current_file_bg)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        filename.clone(),
                        Style::default()
                            .fg(theme.file_tree_current_file_fg)
                            .bg(theme.file_tree_current_file_bg)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        format!(" +{}", additions),
                        Style::default()
                            .fg(theme.file_tree_stats_added)
                            .bg(theme.file_tree_current_file_bg),
                    ));
                    spans.push(Span::styled(
                        format!(" -{}", deletions),
                        Style::default()
                            .fg(theme.file_tree_stats_deleted)
                            .bg(theme.file_tree_current_file_bg),
                    ));

                    // Fill to right edge with background color
                    if content_width < area_width {
                        spans.push(Span::styled(
                            " ".repeat(area_width - content_width),
                            Style::default().bg(theme.file_tree_current_file_bg),
                        ));
                    }
                } else {
                    // Normal file
                    // Add left padding
                    spans.push(Span::styled(
                        "  ",
                        Style::default().bg(theme.background_left),
                    ));
                    spans.push(Span::styled(
                        indent,
                        Style::default().bg(theme.background_left),
                    ));
                    spans.push(Span::styled(
                        status_str,
                        Style::default()
                            .fg(*color)
                            .bg(theme.background_left)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        filename.clone(),
                        Style::default()
                            .fg(theme.file_tree_default)
                            .bg(theme.background_left),
                    ));
                    spans.push(Span::styled(
                        format!(" +{}", additions),
                        Style::default()
                            .fg(theme.file_tree_stats_added)
                            .bg(theme.background_left),
                    ));
                    spans.push(Span::styled(
                        format!(" -{}", deletions),
                        Style::default()
                            .fg(theme.file_tree_stats_deleted)
                            .bg(theme.background_left),
                    ));

                    // Fill to right edge with background color
                    if content_width < area_width {
                        spans.push(Span::styled(
                            " ".repeat(area_width - content_width),
                            Style::default().bg(theme.background_left),
                        ));
                    }
                }

                display_line_count += Self::calculate_wrapped_lines(content_width, area_width);

                lines.push(Line::from(spans));
            }
        }

        (lines, current_line_index, display_line_count)
    }
}
