pub fn language() -> tree_sitter::Language {
    tree_sitter_kotlin_ng::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = include_str!("queries/kotlin_highlights.scm");
