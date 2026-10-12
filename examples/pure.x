import std::io;

pure fn square(value: i32) -> i32 {
    return value * value;
}

fn main() -> i32 {
    let result: i32 = square(7);

    io::print("pure: square(7) = ");
    io::println(result);
    if result != 49 {
        return 1;
    }

    io::println("pure: passed");
    return 0;
}
