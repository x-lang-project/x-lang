import std::io;

struct Counter {
    value: i32,
    invariant { self.value >= 0 }
}

fn main() -> i32 {
    let mut counter: Counter = Counter { value: 0 };
    counter.value = 42;

    io::print("invariant: counter after assignment = ");
    io::println(counter.value);
    if counter.value != 42 {
        return 1;
    }

    io::println("invariant: passed");
    return 0;
}
