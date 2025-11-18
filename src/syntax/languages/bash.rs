pub fn language() -> tree_sitter::Language {
    tree_sitter_bash::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_bash::HIGHLIGHT_QUERY;
