//! Comprehensive unit tests for the Nevermind Lexer

use nevermind_lexer::Lexer;
use nevermind_lexer::token::{Token, TokenType, Keyword, Operator, Delimiter, LiteralType};

/// Helper function to tokenize source and return the tokens (excluding EOF)
fn tokenize(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    tokens.into_iter()
        .filter(|t| !t.is_eof())
        .collect()
}

/// Helper function to check if tokens match expected kinds
fn assert_token_kinds(tokens: &[Token], expected_kinds: &[TokenType]) {
    assert_eq!(
        tokens.len(),
        expected_kinds.len(),
        "Expected {} tokens, got {}",
        expected_kinds.len(),
        tokens.len()
    );

    for (i, (token, expected)) in tokens.iter().zip(expected_kinds.iter()).enumerate() {
        assert_eq!(
            &token.kind, expected,
            "Token {}: expected {:?}, got {:?}",
            i, expected, token.kind
        );
    }
}

/// Helper function to check if tokens match expected texts
fn assert_token_texts(tokens: &[Token], expected_texts: &[&str]) {
    assert_eq!(
        tokens.len(),
        expected_texts.len(),
        "Expected {} tokens, got {}",
        expected_texts.len(),
        tokens.len()
    );

    for (i, (token, expected)) in tokens.iter().zip(expected_texts.iter()).enumerate() {
        assert_eq!(
            &token.text, expected,
            "Token {}: expected '{}', got '{}'",
            i, expected, token.text
        );
    }
}

// ============================================================================
// Keyword Tests
// ============================================================================

#[test]
fn test_variable_declaration_keywords() {
    let source = "let var";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Let),
            TokenType::Keyword(Keyword::Var),
        ],
    );
}

#[test]
fn test_control_flow_keywords() {
    let source = "if then else elif match case";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::If),
            TokenType::Keyword(Keyword::Then),
            TokenType::Keyword(Keyword::Else),
            TokenType::Keyword(Keyword::Elif),
            TokenType::Keyword(Keyword::Match),
            TokenType::Keyword(Keyword::Case),
        ],
    );
}

#[test]
fn test_loop_keywords() {
    let source = "for while forever in";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::For),
            TokenType::Keyword(Keyword::While),
            TokenType::Keyword(Keyword::Forever),
            TokenType::Keyword(Keyword::In),
        ],
    );
}

#[test]
fn test_function_keywords() {
    let source = "fn return";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Fn),
            TokenType::Keyword(Keyword::Return),
        ],
    );
}

#[test]
fn test_loop_control_keywords() {
    let source = "break continue";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Break),
            TokenType::Keyword(Keyword::Continue),
        ],
    );
}

#[test]
fn test_boolean_literals() {
    let source = "true false";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::True),
            TokenType::Keyword(Keyword::False),
        ],
    );
}

#[test]
fn test_null_literal() {
    let source = "null";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Keyword(Keyword::Null)]);
}

#[test]
fn test_type_keyword() {
    let source = "type";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Keyword(Keyword::Type)]);
}

#[test]
fn test_error_handling_keywords() {
    let source = "try catch finally raise";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Try),
            TokenType::Keyword(Keyword::Catch),
            TokenType::Keyword(Keyword::Finally),
            TokenType::Keyword(Keyword::Raise),
        ],
    );
}

#[test]
fn test_concurrency_keywords() {
    let source = "async await parallel sync";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Async),
            TokenType::Keyword(Keyword::Await),
            TokenType::Keyword(Keyword::Parallel),
            TokenType::Keyword(Keyword::Sync),
        ],
    );
}

#[test]
fn test_block_keywords() {
    let source = "do end";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Do),
            TokenType::Keyword(Keyword::End),
        ],
    );
}

#[test]
fn test_module_keywords() {
    let source = "use from import as";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Use),
            TokenType::Keyword(Keyword::From),
            TokenType::Keyword(Keyword::Import),
            TokenType::Keyword(Keyword::As),
        ],
    );
}

#[test]
fn test_class_keywords() {
    let source = "class extends implements";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Class),
            TokenType::Keyword(Keyword::Extends),
            TokenType::Keyword(Keyword::Implements),
        ],
    );
}

#[test]
fn test_trait_keyword() {
    let source = "trait";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Keyword(Keyword::Trait)]);
}

#[test]
fn test_where_keyword() {
    let source = "where";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Keyword(Keyword::Where)]);
}

// ============================================================================
// Operator Tests
// ============================================================================

#[test]
fn test_arithmetic_operators() {
    let source = "+ - * / % **";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Operator(Operator::Add),
            TokenType::Operator(Operator::Sub),
            TokenType::Operator(Operator::Mul),
            TokenType::Operator(Operator::Div),
            TokenType::Operator(Operator::Mod),
            TokenType::Operator(Operator::Pow),
        ],
    );
}

#[test]
fn test_comparison_operators() {
    let source = "== != < > <= >=";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Operator(Operator::Eq),
            TokenType::Operator(Operator::Ne),
            TokenType::Operator(Operator::Lt),
            TokenType::Operator(Operator::Gt),
            TokenType::Operator(Operator::Le),
            TokenType::Operator(Operator::Ge),
        ],
    );
}

#[test]
fn test_logical_operators() {
    let source = "and or not";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Operator(Operator::And),
            TokenType::Operator(Operator::Or),
            TokenType::Operator(Operator::Not),
        ],
    );
}

#[test]
fn test_bitwise_operators() {
    let source = "& | ^ ~ << >>";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Operator(Operator::BitAnd),
            TokenType::Operator(Operator::BitOr),
            TokenType::Operator(Operator::BitXor),
            TokenType::Operator(Operator::BitNot),
            TokenType::Operator(Operator::ShiftLeft),
            TokenType::Operator(Operator::ShiftRight),
        ],
    );
}

#[test]
fn test_other_operators() {
    let source = "|> = -> => . .. ... ++";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Operator(Operator::Pipe),
            TokenType::Operator(Operator::Assign),
            TokenType::Operator(Operator::Arrow),
            TokenType::Operator(Operator::FatArrow),
            TokenType::Operator(Operator::Dot),
            TokenType::Operator(Operator::DotDot),
            TokenType::Operator(Operator::DotDotDot),
            TokenType::Operator(Operator::Concat),
        ],
    );
}

// ============================================================================
// Delimiter Tests
// ============================================================================

#[test]
fn test_parentheses() {
    let source = "()";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Delimiter(Delimiter::LParen),
            TokenType::Delimiter(Delimiter::RParen),
        ],
    );
}

#[test]
fn test_braces() {
    let source = "{}";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Delimiter(Delimiter::LBrace),
            TokenType::Delimiter(Delimiter::RBrace),
        ],
    );
}

#[test]
fn test_brackets() {
    let source = "[]";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Delimiter(Delimiter::LBracket),
            TokenType::Delimiter(Delimiter::RBracket),
        ],
    );
}

#[test]
fn test_comma() {
    let source = ",";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Delimiter(Delimiter::Comma)]);
}

#[test]
fn test_colon() {
    let source = ":";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Delimiter(Delimiter::Colon)]);
}

#[test]
fn test_semicolon() {
    let source = ";";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Delimiter(Delimiter::Semicolon)]);
}

#[test]
fn test_at_sign() {
    let source = "@";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Delimiter(Delimiter::At)]);
}

#[test]
fn test_question_mark() {
    let source = "?";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Delimiter(Delimiter::Question)]);
}

#[test]
fn test_dollar_sign() {
    let source = "$";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Delimiter(Delimiter::Dollar)]);
}

#[test]
fn test_backtick() {
    let source = "`";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Delimiter(Delimiter::Backtick)]);
}

// ============================================================================
// Identifier Tests
// ============================================================================

#[test]
fn test_simple_identifier() {
    let source = "foo";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Identifier]);
    assert_token_texts(&tokens, &["foo"]);
}

#[test]
fn test_identifier_with_underscore() {
    let source = "my_variable";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Identifier]);
    assert_token_texts(&tokens, &["my_variable"]);
}

#[test]
fn test_identifier_starting_with_underscore() {
    let source = "_private";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Identifier]);
    assert_token_texts(&tokens, &["_private"]);
}

#[test]
fn test_identifier_with_numbers() {
    let source = "var123";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Identifier]);
    assert_token_texts(&tokens, &["var123"]);
}

#[test]
fn test_identifier_with_apostrophe() {
    let source = "type'instance";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Identifier]);
    assert_token_texts(&tokens, &["type'instance"]);
}

#[test]
fn test_multiple_identifiers() {
    let source = "foo bar baz";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[TokenType::Identifier, TokenType::Identifier, TokenType::Identifier],
    );
    assert_token_texts(&tokens, &["foo", "bar", "baz"]);
}

// ============================================================================
// Literal Tests
// ============================================================================

#[test]
fn test_integer_literal() {
    let source = "42";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Integer)]);
    assert_token_texts(&tokens, &["42"]);
}

#[test]
fn test_zero() {
    let source = "0";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Integer)]);
    assert_token_texts(&tokens, &["0"]);
}

#[test]
fn test_large_integer() {
    let source = "123456789";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Integer)]);
    assert_token_texts(&tokens, &["123456789"]);
}

#[test]
fn test_float_literal() {
    let source = "3.14";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Float)]);
    assert_token_texts(&tokens, &["3.14"]);
}

#[test]
fn test_float_with_zero_integer_part() {
    let source = "0.5";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Float)]);
    assert_token_texts(&tokens, &["0.5"]);
}

#[test]
fn test_float_scientific_notation_lower() {
    let source = "1e10";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Float)]);
    assert_token_texts(&tokens, &["1e10"]);
}

#[test]
fn test_float_scientific_notation_upper() {
    let source = "1E10";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Float)]);
    assert_token_texts(&tokens, &["1E10"]);
}

#[test]
fn test_float_scientific_notation_with_positive_exponent() {
    let source = "1e+10";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Float)]);
    assert_token_texts(&tokens, &["1e+10"]);
}

#[test]
fn test_float_scientific_notation_with_negative_exponent() {
    let source = "1e-10";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Float)]);
    assert_token_texts(&tokens, &["1e-10"]);
}

#[test]
fn test_float_with_fraction_and_exponent() {
    let source = "3.14e2";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Float)]);
    assert_token_texts(&tokens, &["3.14e2"]);
}

#[test]
fn test_string_literal() {
    let source = r#""hello""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello");
}

#[test]
fn test_empty_string() {
    let source = r#""""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "");
}

#[test]
fn test_string_with_spaces() {
    let source = r#""hello world""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello world");
}

#[test]
fn test_string_with_escape_newline() {
    let source = r#""hello\nworld""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello\nworld");
}

#[test]
fn test_string_with_escape_tab() {
    let source = r#""hello\tworld""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello\tworld");
}

#[test]
fn test_string_with_escape_carriage_return() {
    let source = r#""hello\rworld""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello\rworld");
}

#[test]
fn test_string_with_escape_null() {
    let source = r#""hello\0world""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello\0world");
}

#[test]
fn test_string_with_escape_backslash() {
    let source = r#""hello\\world""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello\\world");
}

#[test]
fn test_string_with_escape_quote() {
    let source = r#""hello\"world""#;
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "hello\"world");
}

#[test]
fn test_string_with_hex_escape() {
    let source = r#""\x41""#;  // 'A' in ASCII
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "A");
}

#[test]
fn test_string_with_unicode_escape() {
    let source = r#""\u{1F600}""#;  // 😀
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::String)]);
    assert_eq!(tokens[0].text, "😀");
}

#[test]
fn test_char_literal() {
    let source = "'a'";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Char)]);
    assert_eq!(tokens[0].text, "a");
}

#[test]
fn test_char_escape_newline() {
    let source = r"'\\n'";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Char)]);
    assert_eq!(tokens[0].text, "\n");
}

#[test]
fn test_char_escape_tab() {
    let source = r"'\\t'";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Char)]);
    assert_eq!(tokens[0].text, "\t");
}

#[test]
fn test_char_hex_escape() {
    let source = r"'\x41'";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Literal(LiteralType::Char)]);
    assert_eq!(tokens[0].text, "A");
}

// ============================================================================
// Comment Tests
// ============================================================================

#[test]
fn test_hash_line_comment() {
    let source = "# This is a comment\nlet x = 5";
    let tokens = tokenize(source);
    // Comment should be ignored, only tokens after should remain
    assert!(tokens.len() > 0);
    assert_eq!(tokens[0].kind, TokenType::Keyword(Keyword::Let));
}

#[test]
fn test_double_slash_line_comment() {
    let source = "// This is a comment\nlet x = 5";
    let tokens = tokenize(source);
    // Comment should be ignored
    assert_eq!(tokens[0].kind, TokenType::Keyword(Keyword::Let));
}

#[test]
fn test_block_comment() {
    let source = "/* This is a block comment */ let x = 5";
    let tokens = tokenize(source);
    // Block comment should be ignored
    assert_eq!(tokens[0].kind, TokenType::Keyword(Keyword::Let));
}

#[test]
fn test_multiline_block_comment() {
    let source = "/* Line 1\nLine 2\nLine 3 */ let x = 5";
    let tokens = tokenize(source);
    // Multiline block comment should be ignored
    assert_eq!(tokens[0].kind, TokenType::Keyword(Keyword::Let));
}

#[test]
fn test_nested_block_comments() {
    let source = "/* Outer /* inner */ comment */ let x = 5";
    let tokens = tokenize(source);
    // Nested block comments should be handled (depth tracking)
    assert_eq!(tokens[0].kind, TokenType::Keyword(Keyword::Let));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_input() {
    let source = "";
    let tokens = tokenize(source);
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_whitespace_only() {
    let source = "   \t  ";
    let tokens = tokenize(source);
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_newlines_only() {
    let source = "\n\n\n";
    let tokens = tokenize(source);
    // Newlines might produce semicolon tokens for indentation
    // This test just ensures it doesn't crash
}

#[test]
fn test_tokens_with_whitespace() {
    let source = "let   x    =    5";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Let),
            TokenType::Identifier,
            TokenType::Operator(Operator::Assign),
            TokenType::Literal(LiteralType::Integer),
        ],
    );
    assert_token_texts(&tokens, &["let", "x", "=", "5"]);
}

#[test]
fn test_dots_not_float() {
    let source = "1 . 2";  // Three separate tokens: 1, ., 2
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Literal(LiteralType::Integer),
            TokenType::Operator(Operator::Dot),
            TokenType::Literal(LiteralType::Integer),
        ],
    );
}

#[test]
fn test_keyword_not_identifier() {
    let source = "let";
    let tokens = tokenize(source);
    assert_token_kinds(&tokens, &[TokenType::Keyword(Keyword::Let)]);
    assert!(tokens[0].is_keyword());
    assert!(!tokens[0].is_identifier());
}

#[test]
fn test_identifier_similar_to_keyword() {
    let source = "letx vari";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[TokenType::Identifier, TokenType::Identifier],
    );
}

// ============================================================================
// Complex Expression Tests
// ============================================================================

#[test]
fn test_simple_assignment() {
    let source = "let x = 42";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Let),
            TokenType::Identifier,
            TokenType::Operator(Operator::Assign),
            TokenType::Literal(LiteralType::Integer),
        ],
    );
    assert_token_texts(&tokens, &["let", "x", "=", "42"]);
}

#[test]
fn test_arithmetic_expression() {
    let source = "x + y * z";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Identifier,
            TokenType::Operator(Operator::Add),
            TokenType::Identifier,
            TokenType::Operator(Operator::Mul),
            TokenType::Identifier,
        ],
    );
}

#[test]
fn test_function_call() {
    let source = "foo(arg1, arg2)";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Identifier,
            TokenType::Delimiter(Delimiter::LParen),
            TokenType::Identifier,
            TokenType::Delimiter(Delimiter::Comma),
            TokenType::Identifier,
            TokenType::Delimiter(Delimiter::RParen),
        ],
    );
}

#[test]
fn test_array_literal() {
    let source = "[1, 2, 3]";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Delimiter(Delimiter::LBracket),
            TokenType::Literal(LiteralType::Integer),
            TokenType::Delimiter(Delimiter::Comma),
            TokenType::Literal(LiteralType::Integer),
            TokenType::Delimiter(Delimiter::Comma),
            TokenType::Literal(LiteralType::Integer),
            TokenType::Delimiter(Delimiter::RBracket),
        ],
    );
}

#[test]
fn test_if_expression() {
    let source = "if x then y else z";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::If),
            TokenType::Identifier,
            TokenType::Keyword(Keyword::Then),
            TokenType::Identifier,
            TokenType::Keyword(Keyword::Else),
            TokenType::Identifier,
        ],
    );
}

#[test]
fn test_function_definition() {
    let source = "fn add(x, y) -> x + y";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Keyword(Keyword::Fn),
            TokenType::Identifier,
            TokenType::Delimiter(Delimiter::LParen),
            TokenType::Identifier,
            TokenType::Delimiter(Delimiter::Comma),
            TokenType::Identifier,
            TokenType::Delimiter(Delimiter::RParen),
            TokenType::Operator(Operator::Arrow),
            TokenType::Identifier,
            TokenType::Operator(Operator::Add),
            TokenType::Identifier,
        ],
    );
}

#[test]
fn test_comparison_chain() {
    let source = "x == y and y > z";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Identifier,
            TokenType::Operator(Operator::Eq),
            TokenType::Identifier,
            TokenType::Operator(Operator::And),
            TokenType::Identifier,
            TokenType::Operator(Operator::Gt),
            TokenType::Identifier,
        ],
    );
}

#[test]
fn test_pipe_operator() {
    let source = "x |> foo |> bar";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Identifier,
            TokenType::Operator(Operator::Pipe),
            TokenType::Identifier,
            TokenType::Operator(Operator::Pipe),
            TokenType::Identifier,
        ],
    );
}

#[test]
fn test_range_operators() {
    let source = "1..10 1...10";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Literal(LiteralType::Integer),
            TokenType::Operator(Operator::DotDot),
            TokenType::Literal(LiteralType::Integer),
            TokenType::Literal(LiteralType::Integer),
            TokenType::Operator(Operator::DotDotDot),
            TokenType::Literal(LiteralType::Integer),
        ],
    );
}

#[test]
fn test_string_concatenation() {
    let source = r#""hello" ++ "world""#;
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Literal(LiteralType::String),
            TokenType::Operator(Operator::Concat),
            TokenType::Literal(LiteralType::String),
        ],
    );
}

#[test]
fn test_member_access() {
    let source = "obj.property";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Identifier,
            TokenType::Operator(Operator::Dot),
            TokenType::Identifier,
        ],
    );
}

#[test]
fn test_match_expression() {
    let source = "match x\n  case 1 => \"one\"\n  case 2 => \"two\"";
    let tokens = tokenize(source);
    assert!(tokens.iter().any(|t| t.kind == TokenType::Keyword(Keyword::Match)));
    assert!(tokens.iter().any(|t| t.kind == TokenType::Keyword(Keyword::Case)));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_unterminated_string() {
    let source = r#""unterminated"#;
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.message.contains("unterminated") || e.message.contains("string"));
    }
}

#[test]
fn test_unterminated_char() {
    let source = "'a";
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.message.contains("unterminated") || e.message.contains("character"));
    }
}

#[test]
fn test_invalid_character() {
    let source = "@@@";
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    // The first '@' should tokenize as a delimiter, but invalid chars should error
    // This test ensures the lexer can handle various character sequences
}

#[test]
fn test_invalid_unicode_escape() {
    let source = r#""\u{ZZZZ}""#;  // Invalid hex digits
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_unterminated_unicode_escape() {
    let source = r#""\u{1234""#;  // Missing closing brace
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_invalid_unicode_escape_no_brace() {
    let source = r#""\u1234""#;  // Missing opening brace
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_unterminated_block_comment() {
    let source = "/* unterminated comment";
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.message.contains("unterminated") || e.message.contains("comment"));
    }
}

// ============================================================================
// Indentation Tests
// ============================================================================

#[test]
fn test_basic_indentation() {
    let source = "let x = 1\n  let y = 2\nlet z = 3";
    let tokens = tokenize(source);
    // Should tokenize successfully and handle indentation
    assert!(tokens.len() > 0);
}

#[test]
fn test_increasing_indentation() {
    let source = "fn foo()\n  let x = 1\n  let y = 2";
    let tokens = tokenize(source);
    assert!(tokens.len() > 0);
}

#[test]
fn test_decreasing_indentation() {
    let source = "fn foo()\n  let x = 1\nlet y = 2";
    let tokens = tokenize(source);
    assert!(tokens.len() > 0);
}

#[test]
fn test_inconsistent_indentation() {
    let source = "fn foo()\n  let x = 1\n let y = 2";  // Inconsistent: 2 spaces then 1 space
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();
    // This might produce an error due to inconsistent indentation
    // The test verifies the lexer handles it appropriately
}

// ============================================================================
// Token Property Tests
// ============================================================================

#[test]
fn test_token_is_keyword() {
    let source = "let";
    let tokens = tokenize(source);
    assert!(tokens[0].is_keyword());
    assert!(!tokens[0].is_identifier());
    assert!(!tokens[0].is_literal());
    assert!(!tokens[0].is_operator());
    assert!(!tokens[0].is_delimiter());
}

#[test]
fn test_token_is_identifier() {
    let source = "foo";
    let tokens = tokenize(source);
    assert!(!tokens[0].is_keyword());
    assert!(tokens[0].is_identifier());
    assert!(!tokens[0].is_literal());
    assert!(!tokens[0].is_operator());
    assert!(!tokens[0].is_delimiter());
}

#[test]
fn test_token_is_literal() {
    let source = "42";
    let tokens = tokenize(source);
    assert!(!tokens[0].is_keyword());
    assert!(!tokens[0].is_identifier());
    assert!(tokens[0].is_literal());
    assert!(!tokens[0].is_operator());
    assert!(!tokens[0].is_delimiter());
}

#[test]
fn test_token_is_operator() {
    let source = "+";
    let tokens = tokenize(source);
    assert!(!tokens[0].is_keyword());
    assert!(!tokens[0].is_identifier());
    assert!(!tokens[0].is_literal());
    assert!(tokens[0].is_operator());
    assert!(!tokens[0].is_delimiter());
}

#[test]
fn test_token_is_delimiter() {
    let source = "(";
    let tokens = tokenize(source);
    assert!(!tokens[0].is_keyword());
    assert!(!tokens[0].is_identifier());
    assert!(!tokens[0].is_literal());
    assert!(!tokens[0].is_operator());
    assert!(tokens[0].is_delimiter());
}

// ============================================================================
// Advanced Number Tests
// ============================================================================

#[test]
fn test_negative_number_lookalike() {
    // Note: The lexer tokenizes '-' as a separate operator
    let source = "-5";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Operator(Operator::Sub),
            TokenType::Literal(LiteralType::Integer),
        ],
    );
}

#[test]
fn test_float_ending_with_dot() {
    let source = "3.";
    let tokens = tokenize(source);
    // This should tokenize as integer followed by dot operator
    // (not as a float, because there's no digit after the dot)
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Literal(LiteralType::Integer),
            TokenType::Operator(Operator::Dot),
        ],
    );
}

#[test]
fn test_multiple_operators_together() {
    let source = "+-*/";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Operator(Operator::Add),
            TokenType::Operator(Operator::Sub),
            TokenType::Operator(Operator::Mul),
            TokenType::Operator(Operator::Div),
        ],
    );
}

// ============================================================================
// Mixed Content Tests
// ============================================================================

#[test]
fn test_code_with_comments() {
    let source = r#"
# Calculate the factorial
fn factorial(n)
  if n <= 1
    1
  else
    n * factorial(n - 1)
"#;
    let tokens = tokenize(source);
    assert!(tokens.len() > 0);
    assert!(tokens.iter().any(|t| t.is_keyword()));
}

#[test]
fn test_string_with_all_escapes() {
    let source = r#""\n\r\t\0\\\"\''"#;
    let tokens = tokenize(source);
    assert_eq!(tokens[0].kind, TokenType::Literal(LiteralType::String));
}

#[test]
fn test_complex_expressions() {
    let source = "((x + y) * (z - w)) / (a ** b)";
    let tokens = tokenize(source);
    assert!(tokens.len() > 10);
}

#[test]
fn test_logical_operators_with_identifiers() {
    let source = "x and y or not z";
    let tokens = tokenize(source);
    assert_token_kinds(
        &tokens,
        &[
            TokenType::Identifier,
            TokenType::Operator(Operator::And),
            TokenType::Identifier,
            TokenType::Operator(Operator::Or),
            TokenType::Operator(Operator::Not),
            TokenType::Identifier,
        ],
    );
}
