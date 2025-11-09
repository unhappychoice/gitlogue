pub fn language() -> tree_sitter::Language {
    tree_sitter_go::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_go::HIGHLIGHTS_QUERY;
