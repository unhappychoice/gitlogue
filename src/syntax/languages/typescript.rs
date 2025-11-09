pub fn language() -> tree_sitter::Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

pub const HIGHLIGHT_QUERY: &str = include_str!("queries/typescript_highlights.scm");
