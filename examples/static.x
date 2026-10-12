import std::io;

fn scale(static factor: i32, value: i32) -> i32 {
    return factor * value;
}

fn main() -> i32 {
    let doubled: i32 = scale(2, 7);
    let tripled: i32 = scale(3, 7);

    io::print("static: scale(2, 7) = ");
    io::println(doubled);
    io::print("static: scale(3, 7) = ");
    io::println(tripled);
    if doubled != 14 || tripled != 21 {
        return 1;
    }

    io::println("static: passed");
    return 0;
}
