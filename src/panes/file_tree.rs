use crate::git::{CommitMetadata, LineChangeType};
use crate::theme::Theme;
use crate::widgets::SelectableParagraph;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding},
    Frame,
};
use std::collections::BTreeMap;

type FileEntry = (usize, String, String, Color, usize, usize);
type FileTree = BTreeMap<String, Vec<FileEntry>>;

pub struct FileTreePane {
    cached_lines: Vec<Line<'static>>,
    cached_current_line_index: Option<usize>,
    cached_metadata_id: Option<String>,
    cached_current_file_index: Option<usize>,
}

impl FileTreePane {
    pub fn new() -> Self {
        Self {
            cached_lines: vec![Line::from("No commit loaded")],
            cached_current_line_index: None,
            cached_metadata_id: None,
            cached_current_file_index: None,
        }
    }

    pub fn set_commit_metadata(
        &mut self,
        metadata: &CommitMetadata,
        current_file_index: usize,
        theme: &Theme,
    ) {
        let metadata_id = metadata.hash.clone();

        // Only recalculate if metadata or current file changed
        if self.cached_metadata_id.as_ref() == Some(&metadata_id)
            && self.cached_current_file_index == Some(current_file_index)
        {
            return;
        }

        let (lines, current_line_index) =
            Self::build_tree_lines(metadata, current_file_index, theme);

        self.cached_lines = lines;
        self.cached_current_line_index = current_line_index;
        self.cached_metadata_id = Some(metadata_id);
        self.cached_current_file_index = Some(current_file_index);
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

        let content = SelectableParagraph::new(self.cached_lines.clone())
            .block(block)
            .selected_line(self.cached_current_line_index)
            .selected_style(Style::default().bg(theme.file_tree_current_file_bg))
            .background_style(Style::default().bg(theme.background_left))
            .padding(Padding::horizontal(2))
            .dim(20, 0.6);
        f.render_widget(content, area);
    }

    fn build_tree_lines(
        metadata: &CommitMetadata,
        current_file_index: usize,
        theme: &Theme,
    ) -> (Vec<Line<'static>>, Option<usize>) {
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
        let sorted_dirs: Vec<_> = tree.keys().cloned().collect();

        for dir in sorted_dirs {
            let mut files = tree.get(&dir).unwrap().clone();
            // Sort files by filename within each directory
            files.sort_by(|a, b| a.1.cmp(&b.1));

            // Add directory header if not root
            if !dir.is_empty() {
                let dir_text = format!("{}/", dir);
                let dir_spans = vec![Span::styled(
                    dir_text,
                    Style::default()
                        .fg(theme.file_tree_directory)
                        .add_modifier(Modifier::BOLD),
                )];
                lines.push(Line::from(dir_spans));
            }

            // Add files
            for (index, filename, status_char, color, additions, deletions) in &files {
                let is_current = *index == current_file_index;

                // Track the line index of the current file (before adding the line)
                if is_current {
                    current_line_index = Some(lines.len());
                }

                let indent = if dir.is_empty() { "" } else { "  " }.to_string();
                let status_str = format!("{} ", status_char);
                let additions_str = format!(" +{}", additions);
                let deletions_str = format!(" -{}", deletions);

                let fg_color = if is_current {
                    theme.file_tree_current_file_fg
                } else {
                    theme.file_tree_default
                };

                let modifier = if is_current {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                };

                let spans = vec![
                    Span::raw(indent),
                    Span::styled(
                        status_str,
                        Style::default().fg(*color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        filename.to_string(),
                        Style::default().fg(fg_color).add_modifier(modifier),
                    ),
                    Span::styled(
                        additions_str,
                        Style::default().fg(theme.file_tree_stats_added),
                    ),
                    Span::styled(
                        deletions_str,
                        Style::default().fg(theme.file_tree_stats_deleted),
                    ),
                ];

                lines.push(Line::from(spans));
            }
        }

        (lines, current_line_index)
    }
}
