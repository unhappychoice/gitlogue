pub fn language() -> tree_sitter::Language {
    tree_sitter_php::LANGUAGE_PHP.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_php::HIGHLIGHTS_QUERY;
