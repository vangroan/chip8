use chip8_compiler::{
    compile::Mapper,
    lex::{debug_print_lexer, Lexer},
    parsing::{CompilationUnit, Parse},
    token_stream::TokenStream,
};

const SOURCE: &str = include_str!("stmts.chip8");

#[test]
fn test_lex_stmts() {
    let lexer = Lexer::new(SOURCE);
    debug_print_lexer(lexer);
}

#[test]
fn test_parse_stmts() {
    let lexer = Lexer::new(SOURCE);
    let mut stream = TokenStream::new(lexer);
    let syntax_node = CompilationUnit::parse(&mut stream).unwrap();
    println!("{:#?}", syntax_node);
}

#[test]
fn test_mapper() {
    let lexer = Lexer::new(SOURCE);
    let mut stream = TokenStream::new(lexer);
    let tree = CompilationUnit::parse(&mut stream).unwrap();
    let symbols = Mapper::new().build_symbols(&tree).unwrap();
    println!("{:#?}", symbols);
}

#[test]
fn test_compile() {}
