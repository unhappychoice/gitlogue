pub fn language() -> tree_sitter::Language {
    tree_sitter_css::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_css::HIGHLIGHTS_QUERY;
