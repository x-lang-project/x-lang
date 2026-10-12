import std::io;

pure fn check_square(index: i32) -> i32 {
    let expected: [i32; 8] = [0, 1, 4, 9, 16, 25, 36, 49];
    let result: i32 = index * index;
    invariant { result == expected[index] }
    return result;
}

fn main() -> i32 {
    io::println("parallel: checking squares for 0..8");
    parallel for index in 0..8 {
        check_square(index);
    }

    io::println("parallel: passed");
    return 0;
}
