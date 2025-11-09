pub fn language() -> tree_sitter::Language {
    tree_sitter_javascript::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_javascript::HIGHLIGHT_QUERY;
