; Minimal Kotlin highlights - compatible with tree-sitter-kotlin-ng

(function_declaration
  name: (identifier) @function)

(call_expression
  (identifier) @function)

(identifier) @variable

(string_literal) @string
(number_literal) @number
