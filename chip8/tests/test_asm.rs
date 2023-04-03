#[test]
fn test_asm_maze() {
    let maze_asm = include_str!("maze.asm");
    let maze_bytes = include_bytes!("../programs/maze");

    match chip8::assemble(maze_asm) {
        Ok(bytecode) => {
            assert_eq!(bytecode, maze_bytes);
        }
        Err(err) => {
            panic!("{}", err)
        }
    }
}
