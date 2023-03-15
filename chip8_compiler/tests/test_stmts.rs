use chip8_compiler::{
    compile::{codegen::CodeGen, Mapper},
    lex::{debug_print_lexer, Lexer},
    parsing::{CompilationUnit, Parse, Prog},
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
    let ast = Prog::parse(&mut stream).unwrap();
    println!("{:#?}", ast);
}

#[test]
fn test_compile_stmts() {
    let lexer = Lexer::new(SOURCE);
    let mut stream = TokenStream::new(lexer);
    let ast = Prog::parse(&mut stream).unwrap();
    let bytecode = CodeGen::new().compile(&ast).unwrap();
    // let symbols = Mapper::new().build_symbols(&tree).unwrap();
    println!("{:#?}", bytecode);
}
