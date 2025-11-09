pub fn language() -> tree_sitter::Language {
    tree_sitter_erlang::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_erlang::HIGHLIGHTS_QUERY;
