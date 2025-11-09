pub fn language() -> tree_sitter::Language {
    tree_sitter_c_sharp::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = include_str!("queries/csharp_highlights.scm");
