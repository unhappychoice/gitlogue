pub fn language() -> tree_sitter::Language {
    tree_sitter_dart::language()
}

pub const HIGHLIGHT_QUERY: &str = include_str!("queries/dart_highlights.scm");
