pub fn language() -> tree_sitter::Language {
    tree_sitter_ruby::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_ruby::HIGHLIGHTS_QUERY;
