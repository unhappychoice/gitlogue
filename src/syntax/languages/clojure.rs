pub fn language() -> tree_sitter::Language {
    tree_sitter_clojure::LANGUAGE.into()
}

pub const HIGHLIGHT_QUERY: &str = include_str!("queries/clojure_highlights.scm");
