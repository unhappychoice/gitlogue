pub mod languages;

use ratatui::style::Color;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

pub use languages::get_language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Type,
    Function,
    Variable,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Constant,
    Parameter,
    Property,
    Label,
}

impl TokenType {
    pub fn color(&self) -> Color {
        match self {
            TokenType::Keyword => Color::Magenta,
            TokenType::Type => Color::Yellow,
            TokenType::Function => Color::Blue,
            TokenType::Variable => Color::White,
            TokenType::String => Color::Green,
            TokenType::Number => Color::Cyan,
            TokenType::Comment => Color::DarkGray,
            TokenType::Operator => Color::LightRed,
            TokenType::Punctuation => Color::White,
            TokenType::Constant => Color::LightYellow,
            TokenType::Parameter => Color::LightCyan,
            TokenType::Property => Color::LightBlue,
            TokenType::Label => Color::LightMagenta,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

pub struct Highlighter {
    parser: Parser,
    language: Option<Language>,
    query: Option<Query>,
    query_source: Option<String>,
    cached_tree: Option<tree_sitter::Tree>,
    cached_source: String,
}

impl Clone for Highlighter {
    fn clone(&self) -> Self {
        let mut new_parser = Parser::new();
        let query = if let (Some(ref lang), Some(ref source)) = (&self.language, &self.query_source) {
            let _ = new_parser.set_language(lang);
            Query::new(lang, source).ok()
        } else {
            None
        };

        Self {
            parser: new_parser,
            language: self.language.clone(),
            query,
            query_source: self.query_source.clone(),
            cached_tree: None,
            cached_source: String::new(),
        }
    }
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            language: None,
            query: None,
            query_source: None,
            cached_tree: None,
            cached_source: String::new(),
        }
    }

    pub fn set_language_from_path(&mut self, path: &str) -> bool {
        if let Some((language, query_source)) = get_language(Path::new(path)) {
            if self.parser.set_language(&language).is_ok() {
                if let Ok(query) = Query::new(&language, query_source) {
                    self.language = Some(language);
                    self.query = Some(query);
                    self.query_source = Some(query_source.to_string());
                    self.cached_tree = None;
                    self.cached_source = String::new();
                    return true;
                }
            }
        }
        // Language not supported - clear previous language settings
        self.language = None;
        self.query = None;
        self.query_source = None;
        self.cached_tree = None;
        self.cached_source = String::new();
        false
    }

    pub fn highlight(&mut self, source: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();

        let Some(query) = &self.query else {
            return spans;
        };

        // Use incremental parsing only if source hasn't changed
        let old_tree = if self.cached_source == source {
            self.cached_tree.as_ref()
        } else {
            None
        };

        let Some(tree) = self.parser.parse(source, old_tree) else {
            return spans;
        };

        // Cache the tree and source for next incremental parse (clone needed because matches borrows tree)
        self.cached_tree = Some(tree.clone());
        self.cached_source = source.to_string();

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());

        loop {
            match matches.next() {
                Some(query_match) => {
                    for capture in query_match.captures {
                        let node = capture.node;
                        let capture_name = &query.capture_names()[capture.index as usize];

                        // Handle dotted capture names like "keyword.function" -> "keyword"
                        let base_name = capture_name.split('.').next().unwrap_or(capture_name);

                        let token_type = match base_name {
                            "keyword" => TokenType::Keyword,
                            "type" => TokenType::Type,
                            "function" => TokenType::Function,
                            "variable" => TokenType::Variable,
                            "string" => TokenType::String,
                            "number" => TokenType::Number,
                            "comment" => TokenType::Comment,
                            "operator" => TokenType::Operator,
                            "punctuation" => TokenType::Punctuation,
                            "constant" => TokenType::Constant,
                            "parameter" => TokenType::Parameter,
                            "property" => TokenType::Property,
                            "label" => TokenType::Label,
                            "character" => TokenType::String,
                            "boolean" => TokenType::Constant,
                            // Additional common capture names
                            "namespace" | "module" => TokenType::Type,
                            "constructor" => TokenType::Type,
                            "method" => TokenType::Function,
                            "macro" => TokenType::Function,
                            "annotation" | "attribute" | "decorator" => TokenType::Keyword,
                            "tag" => TokenType::Type,
                            "escape" => TokenType::Operator,
                            "delimiter" => TokenType::Punctuation,
                            "special" => TokenType::Operator,
                            "field" => TokenType::Property,
                            "enum" | "struct" | "class" | "interface" | "trait" => TokenType::Type,
                            "regexp" => TokenType::String,
                            // Additional from all language queries
                            "conditional" | "repeat" | "exception" | "include" | "storageclass" => TokenType::Keyword,
                            "identifier" => TokenType::Variable,
                            "float" => TokenType::Number,
                            "none" => TokenType::Constant,
                            // Skip internal/special markers
                            "embedded" | "spell" | "__name__" | "_name" | "_op" | "_type" => continue,
                            _ => continue,
                        };

                        spans.push(HighlightSpan {
                            start: node.start_byte(),
                            end: node.end_byte(),
                            token_type,
                        });
                    }
                }
                None => break,
            }
        }

        spans.sort_by_key(|span| span.start);
        spans
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}
