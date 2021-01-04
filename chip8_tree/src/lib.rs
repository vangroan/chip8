//! Tree walking interpreter.
mod nodes;
mod static_tree;
mod utils;

use chip8_core::Chip8Cpu;
pub use static_tree::StaticSimulator;

/// Node in interpreter tree.
pub trait SimNode {
    /// Execute the operation on the given VM state.
    fn exec(&self, cpu: &mut Chip8Cpu);
}
