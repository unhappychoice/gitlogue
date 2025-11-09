pub fn language() -> tree_sitter::Language {
    tree_sitter_cpp::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_cpp::HIGHLIGHT_QUERY;
