use chip8_compiler::lex::{debug_print_lexer, Lexer};

const SOURCE: &str = include_str!("stmts.chip8");

#[test]
fn test_lex_stmts() {
    let lexer = Lexer::new(SOURCE);
    debug_print_lexer(lexer);
}
