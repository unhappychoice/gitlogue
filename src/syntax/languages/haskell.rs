pub fn language() -> tree_sitter::Language {
    tree_sitter_haskell::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_haskell::HIGHLIGHTS_QUERY;
