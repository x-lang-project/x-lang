import std::io;

memoise pure fn square(value: i32) -> i32 {
    return value * value;
}

fn main() -> i32 {
    let first: i32 = square(7);
    let other: i32 = square(8);
    let repeated: i32 = square(7);

    io::print("memoise: square(7) = ");
    io::println(first);
    io::print("memoise: square(8) = ");
    io::println(other);
    io::print("memoise: repeated square(7) = ");
    io::println(repeated);
    if first != 49 || other != 64 || repeated != 49 {
        return 1;
    }

    io::println("memoise: passed");
    return 0;
}
