pub fn language() -> tree_sitter::Language {
    tree_sitter_html::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_html::HIGHLIGHTS_QUERY;
