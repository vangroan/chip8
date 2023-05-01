use criterion::{black_box, criterion_group, criterion_main, Criterion};

use chip8::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    {
        let mut vm = Chip8Vm::new(Chip8Conf::default());
        vm.load_bytecode(include_bytes!("../programs/maze"))
            .unwrap();

        c.bench_function("maze bytecode", |b| {
            b.iter(|| {
                let step_count = black_box(1000_usize);
                // black_box(vm.execute())
                black_box(vm.run_steps(step_count))
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
