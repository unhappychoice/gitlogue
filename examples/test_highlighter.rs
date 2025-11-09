fn main() {
    let test_code = r#"fn main() {
    let x = 42;
    println!("Hello, world!");
}
"#;

    // Parse and print tree
    use tree_sitter::Parser;
    let mut parser = Parser::new();
    let language = gitlogue::syntax::languages::rust::language();
    parser.set_language(&language).unwrap();

    if let Some(tree) = parser.parse(test_code, None) {
        println!("Parse tree:");
        println!("{}", tree.root_node().to_sexp());
    }

    println!("\n--- Testing highlighter ---");
    let mut highlighter = gitlogue::syntax::Highlighter::new();
    let success = highlighter.set_language_from_path("test.rs");
    println!("Language set: {}", success);

    let highlights = highlighter.highlight(test_code);
    println!("Number of highlights: {}", highlights.len());

    for (i, span) in highlights.iter().enumerate() {
        let text = &test_code[span.start..span.end];
        println!(
            "{}: [{}-{}] {:?} = '{}'",
            i, span.start, span.end, span.token_type, text
        );
    }
}
