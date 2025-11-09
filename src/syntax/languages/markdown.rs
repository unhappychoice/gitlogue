pub fn language() -> tree_sitter::Language {
    tree_sitter_md::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_md::HIGHLIGHT_QUERY_BLOCK;
