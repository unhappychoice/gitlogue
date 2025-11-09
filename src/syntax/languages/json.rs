pub fn language() -> tree_sitter::Language {
    tree_sitter_json::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_json::HIGHLIGHTS_QUERY;
