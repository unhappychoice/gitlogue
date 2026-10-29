pub mod astro;
pub mod bash;
pub mod c;
pub mod clojure;
pub mod cpp;
pub mod csharp;
pub mod css;
pub mod dart;
pub mod elixir;
pub mod erlang;
pub mod gdscript;
pub mod gdshader;
pub mod go_lang;
pub mod godot_resource;
pub mod haskell;
pub mod html;
pub mod java;
pub mod javascript;
pub mod json;
pub mod kotlin;
pub mod lua;
pub mod markdown;
pub mod nix;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod svelte;
pub mod swift;
pub mod typescript;
pub mod xml;
pub mod yaml;
pub mod zig;

use std::path::Path;
use tree_sitter::Language;

#[derive(Clone)]
pub struct LanguageSupport {
    pub language: Language,
    pub highlight_query: &'static str,
    pub injection_query: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LanguageKind {
    Astro,
    Bash,
    C,
    Clojure,
    Cpp,
    Csharp,
    Css,
    Dart,
    Elixir,
    Erlang,
    Gdscript,
    Gdshader,
    Go,
    GodotResource,
    Haskell,
    Html,
    Java,
    Javascript,
    Json,
    Kotlin,
    Lua,
    Markdown,
    Nix,
    Php,
    Python,
    Ruby,
    Rust,
    Scala,
    Svelte,
    Swift,
    Typescript,
    Xml,
    Yaml,
    Zig,
}

pub fn get_language(path: &Path) -> Option<LanguageSupport> {
    let extension = path.extension()?.to_str()?;
    by_name(extension)
}

pub fn get_language_by_name(name: &str) -> Option<LanguageSupport> {
    by_name(name.trim().trim_start_matches('.'))
}

fn by_name(raw: &str) -> Option<LanguageSupport> {
    let kind = resolve_language_kind(raw)?;
    Some(match kind {
        LanguageKind::Astro => LanguageSupport {
            language: astro::language(),
            highlight_query: astro::HIGHLIGHT_QUERY,
            injection_query: Some(astro::INJECTION_QUERY),
        },
        LanguageKind::Bash => LanguageSupport {
            language: bash::language(),
            highlight_query: bash::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::C => LanguageSupport {
            language: c::language(),
            highlight_query: c::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Clojure => LanguageSupport {
            language: clojure::language(),
            highlight_query: clojure::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Cpp => LanguageSupport {
            language: cpp::language(),
            highlight_query: cpp::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Csharp => LanguageSupport {
            language: csharp::language(),
            highlight_query: csharp::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Css => LanguageSupport {
            language: css::language(),
            highlight_query: css::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Dart => LanguageSupport {
            language: dart::language(),
            highlight_query: dart::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Elixir => LanguageSupport {
            language: elixir::language(),
            highlight_query: elixir::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Erlang => LanguageSupport {
            language: erlang::language(),
            highlight_query: erlang::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Gdscript => LanguageSupport {
            language: gdscript::language(),
            highlight_query: gdscript::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Gdshader => LanguageSupport {
            language: gdshader::language(),
            highlight_query: gdshader::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Go => LanguageSupport {
            language: go_lang::language(),
            highlight_query: go_lang::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::GodotResource => LanguageSupport {
            language: godot_resource::language(),
            highlight_query: godot_resource::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Haskell => LanguageSupport {
            language: haskell::language(),
            highlight_query: haskell::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Html => LanguageSupport {
            language: html::language(),
            highlight_query: html::HIGHLIGHT_QUERY,
            injection_query: Some(html::INJECTION_QUERY),
        },
        LanguageKind::Java => LanguageSupport {
            language: java::language(),
            highlight_query: java::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Javascript => LanguageSupport {
            language: javascript::language(),
            highlight_query: javascript::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Json => LanguageSupport {
            language: json::language(),
            highlight_query: json::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Kotlin => LanguageSupport {
            language: kotlin::language(),
            highlight_query: kotlin::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Lua => LanguageSupport {
            language: lua::language(),
            highlight_query: lua::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Markdown => LanguageSupport {
            language: markdown::language(),
            highlight_query: markdown::HIGHLIGHT_QUERY,
            injection_query: Some(markdown::INJECTION_QUERY),
        },
        LanguageKind::Nix => LanguageSupport {
            language: nix::language(),
            highlight_query: nix::HIGHLIGHT_QUERY,
            injection_query: Some(nix::INJECTION_QUERY),
        },
        LanguageKind::Php => LanguageSupport {
            language: php::language(),
            highlight_query: php::HIGHLIGHT_QUERY,
            injection_query: Some(php::INJECTION_QUERY),
        },
        LanguageKind::Python => LanguageSupport {
            language: python::language(),
            highlight_query: python::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Ruby => LanguageSupport {
            language: ruby::language(),
            highlight_query: ruby::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Rust => LanguageSupport {
            language: rust::language(),
            highlight_query: rust::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Scala => LanguageSupport {
            language: scala::language(),
            highlight_query: scala::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Svelte => LanguageSupport {
            language: svelte::language(),
            highlight_query: svelte::HIGHLIGHT_QUERY,
            injection_query: Some(svelte::INJECTION_QUERY),
        },
        LanguageKind::Swift => LanguageSupport {
            language: swift::language(),
            highlight_query: swift::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Typescript => LanguageSupport {
            language: typescript::language(),
            highlight_query: typescript::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Xml => LanguageSupport {
            language: xml::language(),
            highlight_query: xml::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Yaml => LanguageSupport {
            language: yaml::language(),
            highlight_query: yaml::HIGHLIGHT_QUERY,
            injection_query: None,
        },
        LanguageKind::Zig => LanguageSupport {
            language: zig::language(),
            highlight_query: zig::HIGHLIGHT_QUERY,
            injection_query: None,
        },
    })
}

#[cfg(test)]
fn canonicalize(raw: &str) -> Option<&'static str> {
    resolve_language_kind(raw).map(LanguageKind::as_str)
}

fn resolve_language_kind(raw: &str) -> Option<LanguageKind> {
    let lower = raw.to_ascii_lowercase();
    match lower.as_str() {
        "astro" => Some(LanguageKind::Astro),
        "sh" | "bash" | "zsh" | "shell" => Some(LanguageKind::Bash),
        "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" | "tcc" | "inl" => {
            Some(LanguageKind::Cpp)
        }
        "c" | "h" => Some(LanguageKind::C),
        "clj" | "cljs" | "cljc" | "edn" | "clojure" => Some(LanguageKind::Clojure),
        "cs" | "csx" | "csharp" | "c#" => Some(LanguageKind::Csharp),
        "css" | "scss" | "postcss" | "less" => Some(LanguageKind::Css),
        "dart" => Some(LanguageKind::Dart),
        "ex" | "exs" | "elixir" => Some(LanguageKind::Elixir),
        "erl" | "hrl" | "es" | "escript" | "erlang" => Some(LanguageKind::Erlang),
        "gd" | "gdscript" => Some(LanguageKind::Gdscript),
        "gdshader" | "gdshaderinc" => Some(LanguageKind::Gdshader),
        "go" | "golang" => Some(LanguageKind::Go),
        "tscn" | "tres" | "godot" | "godot-resource" => Some(LanguageKind::GodotResource),
        "hs" | "lhs" | "haskell" => Some(LanguageKind::Haskell),
        "html" | "htm" => Some(LanguageKind::Html),
        "java" => Some(LanguageKind::Java),
        "js" | "jsx" | "mjs" | "cjs" | "javascript" => Some(LanguageKind::Javascript),
        "json" | "jsonc" | "json5" => Some(LanguageKind::Json),
        "kt" | "kts" | "kotlin" => Some(LanguageKind::Kotlin),
        "lua" => Some(LanguageKind::Lua),
        "md" | "markdown" => Some(LanguageKind::Markdown),
        "nix" => Some(LanguageKind::Nix),
        "php" | "php3" | "php4" | "php5" | "phtml" => Some(LanguageKind::Php),
        "py" | "pyw" | "python" => Some(LanguageKind::Python),
        "rb" | "rbw" | "rake" | "gemspec" | "ruby" => Some(LanguageKind::Ruby),
        "rs" | "rust" => Some(LanguageKind::Rust),
        "scala" | "sc" | "sbt" => Some(LanguageKind::Scala),
        "svelte" => Some(LanguageKind::Svelte),
        "swift" => Some(LanguageKind::Swift),
        "ts" | "tsx" | "mts" | "cts" | "typescript" => Some(LanguageKind::Typescript),
        "xml" | "svg" | "xsl" | "xslt" => Some(LanguageKind::Xml),
        "yaml" | "yml" => Some(LanguageKind::Yaml),
        "zig" => Some(LanguageKind::Zig),
        _ => None,
    }
}

#[cfg(test)]
impl LanguageKind {
    fn as_str(self) -> &'static str {
        match self {
            LanguageKind::Astro => "astro",
            LanguageKind::Bash => "bash",
            LanguageKind::C => "c",
            LanguageKind::Clojure => "clojure",
            LanguageKind::Cpp => "cpp",
            LanguageKind::Csharp => "csharp",
            LanguageKind::Css => "css",
            LanguageKind::Dart => "dart",
            LanguageKind::Elixir => "elixir",
            LanguageKind::Erlang => "erlang",
            LanguageKind::Gdscript => "gdscript",
            LanguageKind::Gdshader => "gdshader",
            LanguageKind::Go => "go",
            LanguageKind::GodotResource => "godot-resource",
            LanguageKind::Haskell => "haskell",
            LanguageKind::Html => "html",
            LanguageKind::Java => "java",
            LanguageKind::Javascript => "javascript",
            LanguageKind::Json => "json",
            LanguageKind::Kotlin => "kotlin",
            LanguageKind::Lua => "lua",
            LanguageKind::Markdown => "markdown",
            LanguageKind::Nix => "nix",
            LanguageKind::Php => "php",
            LanguageKind::Python => "python",
            LanguageKind::Ruby => "ruby",
            LanguageKind::Rust => "rust",
            LanguageKind::Scala => "scala",
            LanguageKind::Svelte => "svelte",
            LanguageKind::Swift => "swift",
            LanguageKind::Typescript => "typescript",
            LanguageKind::Xml => "xml",
            LanguageKind::Yaml => "yaml",
            LanguageKind::Zig => "zig",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::{Parser, Query};

    const SUPPORTED_LANGUAGES: [(&str, bool); 34] = [
        ("astro", true),
        ("bash", false),
        ("c", false),
        ("clojure", false),
        ("cpp", false),
        ("csharp", false),
        ("css", false),
        ("dart", false),
        ("elixir", false),
        ("erlang", false),
        ("gdscript", false),
        ("gdshader", false),
        ("go", false),
        ("godot-resource", false),
        ("haskell", false),
        ("html", true),
        ("java", false),
        ("javascript", false),
        ("json", false),
        ("kotlin", false),
        ("lua", false),
        ("markdown", true),
        ("nix", true),
        ("php", true),
        ("python", false),
        ("ruby", false),
        ("rust", false),
        ("scala", false),
        ("svelte", true),
        ("swift", false),
        ("typescript", false),
        ("xml", false),
        ("yaml", false),
        ("zig", false),
    ];

    const CANONICAL_ALIASES: [(&str, &str); 34] = [
        ("astro", "astro"),
        ("shell", "bash"),
        ("h", "c"),
        ("cljc", "clojure"),
        ("c++", "cpp"),
        ("c#", "csharp"),
        ("scss", "css"),
        ("dart", "dart"),
        ("exs", "elixir"),
        ("escript", "erlang"),
        ("gd", "gdscript"),
        ("gdshaderinc", "gdshader"),
        ("golang", "go"),
        ("tscn", "godot-resource"),
        ("lhs", "haskell"),
        ("htm", "html"),
        ("java", "java"),
        ("cjs", "javascript"),
        ("json5", "json"),
        ("kts", "kotlin"),
        ("lua", "lua"),
        ("md", "markdown"),
        ("nix", "nix"),
        ("phtml", "php"),
        ("pyw", "python"),
        ("gemspec", "ruby"),
        ("rs", "rust"),
        ("sbt", "scala"),
        ("svelte", "svelte"),
        ("swift", "swift"),
        ("tsx", "typescript"),
        ("xslt", "xml"),
        ("yml", "yaml"),
        ("zig", "zig"),
    ];

    fn query_signature(name: &str) -> (&'static str, Option<&'static str>) {
        let support = get_language_by_name(name).expect("language should resolve");
        (support.highlight_query, support.injection_query)
    }

    #[test]
    fn every_supported_language_loads_into_parser() {
        SUPPORTED_LANGUAGES
            .iter()
            .for_each(|(name, has_injection)| {
                let support = get_language_by_name(name).expect("language should resolve");
                let mut parser = Parser::new();
                assert!(parser.set_language(&support.language).is_ok(), "{name}");
                assert!(!support.highlight_query.is_empty(), "{name}");
                assert_eq!(support.injection_query.is_some(), *has_injection, "{name}");
            });
    }

    #[test]
    fn every_supported_language_query_compiles() {
        SUPPORTED_LANGUAGES.iter().for_each(|(name, _)| {
            let support = get_language_by_name(name).expect("language should resolve");
            let highlight = Query::new(&support.language, support.highlight_query);
            assert!(highlight.is_ok(), "{name}: {:?}", highlight.err());
            support.injection_query.iter().for_each(|source| {
                let injection = Query::new(&support.language, source);
                assert!(injection.is_ok(), "{name}: {:?}", injection.err());
            });
        });
    }

    #[test]
    fn aliases_resolve_to_the_same_language_support() {
        CANONICAL_ALIASES.iter().for_each(|(alias, canonical)| {
            assert_eq!(canonicalize(alias), Some(*canonical));
            assert_eq!(
                query_signature(alias),
                query_signature(canonical),
                "{alias}"
            );
        });
    }

    #[test]
    fn get_language_by_name_trims_whitespace_and_leading_dots() {
        [
            (" .TSX ", "typescript"),
            ("\t.C#\n", "csharp"),
            ("  .MD  ", "markdown"),
        ]
        .iter()
        .for_each(|(raw, canonical)| {
            assert_eq!(query_signature(raw), query_signature(canonical), "{raw}");
        });
    }

    #[test]
    fn get_language_uses_path_extensions_and_rejects_unknown_inputs() {
        [
            ("src/main.rs", "rust"),
            ("styles/site.scss", "css"),
            ("components/App.tsx", "typescript"),
            ("templates/index.phtml", "php"),
            ("scripts/player.gd", "gdscript"),
            ("shaders/water.gdshaderinc", "gdshader"),
            ("scenes/main.tscn", "godot-resource"),
            ("resources/player.tres", "godot-resource"),
            ("game/project.godot", "godot-resource"),
            ("docs/guide.md", "markdown"),
        ]
        .iter()
        .for_each(|(path, canonical)| {
            let support = get_language(Path::new(path)).expect("path should resolve");
            assert_eq!(
                (support.highlight_query, support.injection_query),
                query_signature(canonical),
                "{path}"
            );
        });

        ["Dockerfile", "archive.tar.gz", "notes.custom"]
            .iter()
            .for_each(|path| assert!(get_language(Path::new(path)).is_none(), "{path}"));
    }

    #[cfg(unix)]
    #[test]
    fn get_language_rejects_non_utf8_extensions() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = std::path::PathBuf::from(OsString::from_vec(b"src/file.\xFF".to_vec()));

        assert!(get_language(&path).is_none());
    }

    #[test]
    fn unknown_language_names_return_none() {
        ["", ".", "unknown", "gitlogue"]
            .iter()
            .for_each(|name| assert!(get_language_by_name(name).is_none(), "{name}"));
    }
}
