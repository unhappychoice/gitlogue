pub fn language() -> tree_sitter::Language {
    tree_sitter_swift::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = include_str!("queries/swift_highlights.scm");
