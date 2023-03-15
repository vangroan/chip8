use criterion::{black_box, criterion_group, criterion_main, Criterion};

use chip8::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    {
        let interpreter = BytecodeInterp;
        let mut vm = Chip8Vm::new(interpreter);
        vm.load_bytecode(include_bytes!("../programs/maze"));

        c.bench_function("maze bytecode", |b| {
            b.iter(|| {
                let _ = black_box(1000);
                vm.execute()
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
