use chip8::prelude::*;

#[test]
fn test_disassemblerv2() {
    const ROM: &[u8] = include_bytes!("maze.rom");
    let mut disasm = DisassemblerV2::new(ROM);

    let mut buf = String::new();
    disasm.disassemble(&mut buf).unwrap();
    println!("{buf}");
}
