pub mod compile;
pub mod lex;
pub mod parsing;
pub mod token_stream;
pub mod tokens;
pub mod trivia;

use parsing::Parse;

pub fn compile_str(source: &str) -> Result<Vec<u8>, compile::CompileError> {
    // Lexical analysis
    let lexer = lex::Lexer::new(source);
    let mut stream = token_stream::TokenStream::new(lexer);

    // Syntactic analysis
    let tree = parsing::CompilationUnit::parse(&mut stream).unwrap();

    // Semantic analysis
    let symbols = compile::Mapper::new().build_symbols(&tree).unwrap();

    // Code generation
    let bytecode = compile::CodeGen::new().compile(&tree, &symbols)?;

    Ok(bytecode)
}
