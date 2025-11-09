pub fn language() -> tree_sitter::Language {
    tree_sitter_scala::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_scala::HIGHLIGHTS_QUERY;
