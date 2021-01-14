//! Opcode implementations.
use crate::SimNode;
use chip8_core::{prelude::*, Address, DISPLAY_BUF_COUNT, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use rand::{prelude::*, thread_rng};

pub enum OpCode {
    NoOp,
    Jump(Jump),
    SkipEqual(SeNode),
    Load(LdNode),
    Add(AddNode),
    LoadAddress(LdINode),
    Random(RndNode),
    Draw(DrwNode),
}

/// 1NNN
/// Jump to address.
pub struct Jump {
    pub address: Address,
}

impl SimNode for Jump {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.pc = self.address as usize;
    }
}

/// 3XNN
/// Skip the next instruction if register VX equals value NN.
pub struct SeNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for SeNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        if cpu.registers[self.vx as usize] == self.nn {
            cpu.pc += 4;
        } else {
            cpu.pc += 2;
        }
    }
}

/// 6XNN
/// Set register VX to value NN.
pub struct LdNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for LdNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.registers[self.vx as usize] = self.nn;
        cpu.pc += 2;
    }
}

/// 7XNN
/// Add value NN to register VX. Carry flag is not set.
pub struct AddNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for AddNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.registers[self.vx as usize] += self.nn;
        cpu.pc += 2;
    }
}

/// ANNN
/// Set address register I to value NNN.
pub struct LdINode {
    pub nnn: u16,
}

impl SimNode for LdINode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.address = self.nnn;
        cpu.pc += 2;
    }
}

/// CXNN
/// Set register VX to the result of bitwise AND between a random number and NN.
pub struct RndNode {
    pub vx: u8,
    pub nn: u8,
}

impl SimNode for RndNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        cpu.registers[self.vx as usize] = self.nn & thread_rng().gen::<u8>();
        cpu.pc += 2;
    }
}

/// DXYN
///
/// Draw sprite in the display buffer, at coordinate as per registers VX and VY.
/// Sprite is encoded as 8 pixels wide, N+1 pixels high, stored in bits located in
/// memory pointed to by address register I.
///
/// If the sprite is drawn outside of the display area, it is wrapped around to the other side.
///
/// If the drawing operation erases existing pixels in the display buffer, register VF is set to
/// 1, and set to 0 if no display bits are unset. This is used for collision detection.
pub struct DrwNode {
    pub vx: u8,
    pub vy: u8,
    pub n: u8,
}

impl SimNode for DrwNode {
    #[inline]
    fn exec(&self, cpu: &mut Chip8Cpu) {
        let (x, y) = (
            cpu.registers[self.vx as usize] as usize,
            cpu.registers[self.vy as usize] as usize,
        );
        let mut is_erased = false;

        // Iteration from pointer in address register I to number of rows specified by opcode value N.
        for (r, row) in cpu
            .ram
            .iter()
            .skip(cpu.address as usize)
            .take(self.n as usize)
            .enumerate()
        {
            // Each row is 8 bits representing the 8 pixels of the sprite.
            for c in 0..8 {
                let d = ((x + c) % DISPLAY_WIDTH) + ((y + r) % DISPLAY_HEIGHT) * DISPLAY_WIDTH;

                let old_px = cpu.display[d];
                let new_px = old_px ^ ((row >> (7 - c) & 0x1) == 1);

                // XOR erases a pixel when both the old and new values are both 1.
                is_erased |= old_px && new_px;

                // Write to display buffer
                cpu.display[d] = new_px;
            }
        }

        // If a pixel was erased, then a collision occurred.
        cpu.registers[0xF] = is_erased as u8;
        cpu.pc += 2;
    }
}

// =====================================================================================================================

pub type Sprite = [u8; 4];

pub struct ExecutionContext {
    registers: [u8; 16],
    data: [Sprite; 2],
    display: [bool; DISPLAY_BUF_COUNT],
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            registers: [0; 16],
            data: [[0; 4]; 2],
            display: [false; DISPLAY_BUF_COUNT],
        }
    }

    pub fn dump_display(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write as FmtWrite;

        let mut buf = String::new();

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.display[x + y * DISPLAY_WIDTH] {
                    write!(buf, "#")?;
                } else {
                    write!(buf, ".")?;
                }
            }
            write!(buf, "\n")?;
        }

        Ok(buf)
    }
}

pub struct CompiledExpr<'s>(Box<dyn 's + Fn(&mut ExecutionContext)>);

impl<'s> CompiledExpr<'s> {
    pub fn new(closure: impl 's + Fn(&mut ExecutionContext)) -> Self {
        CompiledExpr(Box::new(closure))
    }

    pub fn execute(&self, ctx: &mut ExecutionContext) {
        self.0(ctx)
    }
}

///
/// ```python
/// sprite1 = ...
/// sprite2 = ...
/// for y in range(32):
///     for x in range(64):
///         if random(1) == 1:
///             draw(sprite1)
///         else:
///             draw(sprite2)
/// ```
pub fn compile_maze<'s>() -> CompiledExpr<'s> {
    // Register labels to make it clear that these values can be determined during AST->Sim compilation.
    const V0: usize = 0;
    const V1: usize = 1;
    const V2: usize = 2;
    const V3: usize = 3;

    // These sprites are data in the maze program's bytecode.
    //
    // In a hypothetical language compiled to an AST, they could be
    // literals of some kind. The arrays or vectors would be created
    // by the AST->Sim compilation and passed to the closures.
    const SPRITE1: Sprite = [0b10000000, 0b01000000, 0b00100000, 0b00010000];
    const SPRITE2: Sprite = [0b00100000, 0b01000000, 0b10000000, 0b00010000];

    let draw = CompiledExpr::new(move |ctx| {
        let (x, y) = (
            ctx.registers[V0 as usize] as usize,
            ctx.registers[V1 as usize] as usize,
        );
        // Argument passing in a real implementation would have to be far more flexible.
        let sprite = ctx.data[ctx.registers[V3] as usize];
        let mut is_erased = false;

        // Iteration from pointer in address register I to number of rows specified by opcode value N.
        for (r, row) in sprite.iter().enumerate() {
            // Each row is 8 bits representing the 8 pixels of the sprite.
            for c in 0..8 {
                let d = ((x + c) % DISPLAY_WIDTH) + ((y + r) % DISPLAY_HEIGHT) * DISPLAY_WIDTH;

                let old_px = ctx.display[d];
                let new_px = old_px ^ ((row >> (7 - c) & 0x1) == 1);

                // XOR erases a pixel when both the old and new values are both 1.
                is_erased |= old_px && new_px;

                // Write to display buffer
                ctx.display[d] = new_px;
            }
        }

        // If a pixel was erased, then a collision occurred.
        ctx.registers[0xF] = is_erased as u8;
    });

    let rnd = CompiledExpr::new(move |ctx| {
        // The 1 in this expression would come from an integer literal in the AST.
        ctx.registers[V2] = 1 & thread_rng().gen::<u8>();
    });

    let cond = CompiledExpr::new(move |ctx| {
        rnd.execute(ctx);

        // The 1 in this expression would come from an integer literal in the AST.
        if ctx.registers[V2] == 1 {
            // Argument passing in a real implementation would have to be far more flexible.
            ctx.registers[V3] = 0;
            draw.execute(ctx);
        } else {
            ctx.registers[V3] = 1;
            draw.execute(ctx);
        }
    });

    let x_loop = CompiledExpr::new(move |ctx| {
        for x in (0..64).step_by(4) {
            // Loop binds a variable to the local scope.
            //
            // Available to expression within the scope.
            ctx.registers[V0] = x;
            cond.execute(ctx);
        }
    });

    let y_loop = CompiledExpr::new(move |ctx| {
        for y in (0..32).step_by(4) {
            // Loop binds a variable to the local scope.
            //
            // Available to expression within the scope.
            ctx.registers[V1] = y;
            x_loop.execute(ctx);
        }
    });

    let set_data = CompiledExpr::new(move |ctx| {
        ctx.data[0] = SPRITE1;
        ctx.data[1] = SPRITE2;
    });

    let root = CompiledExpr::new(move |ctx| {
        set_data.execute(ctx);
        y_loop.execute(ctx);
    });

    root
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_sim() {
        let mut ctx = ExecutionContext::new();
        let root = compile_maze();

        let start = Instant::now();
        root.execute(&mut ctx);
        let end = Instant::now();

        println!(
            "time taken: {}ms",
            end.duration_since(start).as_nanos() as f64 / 1000000.0
        ); // to millis
        println!("{}", ctx.dump_display().unwrap());
    }
}
