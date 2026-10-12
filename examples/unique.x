import std::io;

fn increment(unique value: &mut i32) {
    *value = *value + 1;
}

fn main() -> i32 {
    let mut value: i32 = 41;
    increment(&unique mut value);

    io::print("unique: value after increment = ");
    io::println(value);
    if value != 42 {
        return 1;
    }

    io::println("unique: passed");
    return 0;
}
