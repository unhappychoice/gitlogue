pub fn language() -> tree_sitter::Language {
    tree_sitter_dart::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_dart::HIGHLIGHTS_QUERY;
