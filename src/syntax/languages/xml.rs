pub fn language() -> tree_sitter::Language {
    tree_sitter_xml::LANGUAGE_XML.into()
}

pub const HIGHLIGHT_QUERY: &str = tree_sitter_xml::XML_HIGHLIGHT_QUERY;
