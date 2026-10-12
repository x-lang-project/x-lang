import std::io;

fn count(remaining: i32, total: i32) -> i32 {
    if remaining == 0 {
        return total;
    }
    become count(remaining - 1, total + 1);
}

fn main() -> i32 {
    let result: i32 = count(100000, 0);

    io::print("become: completed tail calls = ");
    io::println(result);
    if result != 100000 {
        return 1;
    }

    io::println("become: passed");
    return 0;
}
