pub fn language() -> tree_sitter::Language {
    tree_sitter_elixir::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_elixir::HIGHLIGHTS_QUERY;
