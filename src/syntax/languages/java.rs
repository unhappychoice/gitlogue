pub fn language() -> tree_sitter::Language {
    tree_sitter_java::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_java::HIGHLIGHTS_QUERY;
