import std::io;

multi fn double(value: i32) -> i32 {
    return value * 2;
}

multi fn double(value: f64) -> f64 {
    return value * 2.0;
}

fn main() -> i32 {
    let integer: i32 = double(21);
    let decimal: f64 = double(1.5);

    io::print("multi: double(21) = ");
    io::println(integer);
    io::print("multi: double(1.5) = ");
    io::println(decimal);
    if integer != 42 || decimal != 3.0 {
        return 1;
    }

    io::println("multi: passed");
    return 0;
}
