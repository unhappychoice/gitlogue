pub mod bash;
pub mod c;
pub mod clojure;
pub mod cpp;
pub mod csharp;
pub mod css;
pub mod dart;
pub mod elixir;
pub mod erlang;
pub mod go_lang;
pub mod haskell;
pub mod html;
pub mod java;
pub mod javascript;
pub mod json;
pub mod kotlin;
pub mod markdown;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod swift;
pub mod typescript;
pub mod xml;
pub mod yaml;
pub mod zig;

use std::path::Path;
use tree_sitter::Language;

pub fn get_language(path: &Path) -> Option<(Language, &'static str)> {
    let extension = path.extension()?.to_str()?;

    match extension {
        "rs" => Some((rust::language(), rust::HIGHLIGHT_QUERY)),
        "ts" | "tsx" | "mts" | "cts" => Some((typescript::language(), typescript::HIGHLIGHT_QUERY)),
        "js" | "jsx" | "mjs" | "cjs" => Some((javascript::language(), javascript::HIGHLIGHT_QUERY)),
        "py" | "pyw" => Some((python::language(), python::HIGHLIGHT_QUERY)),
        "sh" | "bash" | "zsh" => Some((bash::language(), bash::HIGHLIGHT_QUERY)),
        "go" => Some((go_lang::language(), go_lang::HIGHLIGHT_QUERY)),
        "rb" | "rbw" | "rake" | "gemspec" => Some((ruby::language(), ruby::HIGHLIGHT_QUERY)),
        "swift" => Some((swift::language(), swift::HIGHLIGHT_QUERY)),
        "kt" | "kts" => Some((kotlin::language(), kotlin::HIGHLIGHT_QUERY)),
        "java" => Some((java::language(), java::HIGHLIGHT_QUERY)),
        "php" | "php3" | "php4" | "php5" | "phtml" => Some((php::language(), php::HIGHLIGHT_QUERY)),
        "cs" | "csx" => Some((csharp::language(), csharp::HIGHLIGHT_QUERY)),
        // C++ before C to handle .h files (can be either)
        "cpp" | "cc" | "cxx" | "c++" | "C" | "CPP" | "hpp" | "hh" | "hxx" | "h++" | "H" | "HPP"
        | "tcc" | "inl" => Some((cpp::language(), cpp::HIGHLIGHT_QUERY)),
        "c" | "h" => Some((c::language(), c::HIGHLIGHT_QUERY)),
        "hs" | "lhs" => Some((haskell::language(), haskell::HIGHLIGHT_QUERY)),
        "dart" => Some((dart::language(), dart::HIGHLIGHT_QUERY)),
        "scala" | "sc" | "sbt" => Some((scala::language(), scala::HIGHLIGHT_QUERY)),
        "clj" | "cljs" | "cljc" | "edn" => Some((clojure::language(), clojure::HIGHLIGHT_QUERY)),
        "zig" => Some((zig::language(), zig::HIGHLIGHT_QUERY)),
        "ex" | "exs" => Some((elixir::language(), elixir::HIGHLIGHT_QUERY)),
        "erl" | "hrl" | "es" | "escript" => Some((erlang::language(), erlang::HIGHLIGHT_QUERY)),
        "html" | "htm" => Some((html::language(), html::HIGHLIGHT_QUERY)),
        "css" | "scss" | "sass" => Some((css::language(), css::HIGHLIGHT_QUERY)),
        "json" | "jsonc" => Some((json::language(), json::HIGHLIGHT_QUERY)),
        "md" | "markdown" => Some((markdown::language(), markdown::HIGHLIGHT_QUERY)),
        "yaml" | "yml" => Some((yaml::language(), yaml::HIGHLIGHT_QUERY)),
        "xml" | "svg" | "xsl" | "xslt" => Some((xml::language(), xml::HIGHLIGHT_QUERY)),
        _ => None,
    }
}
