


// -*- rust -*-
fn f() -> int { ret 42; }

fn main() {
    let g: extern fn() -> int = f;
    let i: int = g();
    assert (i == 42);
}
