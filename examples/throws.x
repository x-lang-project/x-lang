import std::io;

throws fn divide(total: i32, count: i32) -> i32 {
    if count == 0 {
        throw "Division by zero";
    }
    return total / count;
}

throws fn average(total: i32, count: i32) -> i32 {
    return divide(total, count)?;
}

fn main() -> i32 {
    let success: i32 = catch(average(84, 2), -1);
    let recovered: i32 = catch(average(84, 0), -1);

    io::print("throws: average(84, 2) = ");
    io::println(success);
    io::print("throws: recovered average(84, 0) = ");
    io::println(recovered);
    if success != 42 || recovered != -1 {
        return 1;
    }

    io::println("throws: passed");
    return 0;
}
