use criterion::{black_box, criterion_group, criterion_main, Criterion};

use chip8::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    {
        let mut vm = Chip8Vm::new();
        vm.load_bytecode(include_bytes!("../programs/maze"))
            .unwrap();

        c.bench_function("maze bytecode", |b| {
            b.iter(|| {
                let _ = black_box(1000);
                black_box(vm.execute())
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
