use chip8_bytecode::BytecodeInterpreter;
use chip8_core::prelude::*;
use chip8_tree::{compile_maze, ExecutionContext};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    {
        let interpreter = BytecodeInterpreter;
        let mut vm = Chip8Vm::new(interpreter);
        vm.load_bytecode(include_bytes!("../programs/maze"));

        c.bench_function("maze bytecode", |b| {
            b.iter(|| {
                let _ = black_box(1000);
                vm.execute()
            })
        });
    }

    {
        let mut ctx = ExecutionContext::new();
        let root = compile_maze();

        c.bench_function("maze tree", |b| {
            b.iter(|| {
                let _ = black_box(1000);
                root.execute(&mut ctx)
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
