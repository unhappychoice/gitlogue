pub fn language() -> tree_sitter::Language {
    tree_sitter_yaml::language()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_yaml::HIGHLIGHTS_QUERY;
