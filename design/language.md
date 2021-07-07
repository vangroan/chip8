
```
#include "other.chip8"

var x = 1 + 2 * 3;

def foobar(a: u8, b: u8) -> u8 {
    return a + b;
}

const a = sprite {
    1 0 0 0
    0 1 0 0
    0 0 1 0
    0 0 0 1
};

const b = sprite {
    0 0 0 1
    0 0 1 0
    0 1 0 0
    1 0 0 0
};

def gen_maze() {
    for y in 0..32 {
        for x in 0..64 {
            if random(1) == 1 {
                draw(x, y, a);
            } else {
                draw(x, y, b);
            }
        }
    }
}

struct Foo {
    a: u8,
    b: bool,
}

def optional(a: &u8, b: &Foo) -> u8 {
    if b.b {
        return a + b.a;
    } else {
        return a - b.a;
    }
}
```
