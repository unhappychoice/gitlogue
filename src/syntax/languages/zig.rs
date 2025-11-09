pub fn language() -> tree_sitter::Language {
    tree_sitter_zig::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_zig::HIGHLIGHTS_QUERY;
