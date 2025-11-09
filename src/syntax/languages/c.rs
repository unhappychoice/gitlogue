pub fn language() -> tree_sitter::Language {
    tree_sitter_c::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_c::HIGHLIGHT_QUERY;
