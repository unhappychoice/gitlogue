pub fn language() -> tree_sitter::Language {
    tree_sitter_rust::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_rust::HIGHLIGHTS_QUERY;
