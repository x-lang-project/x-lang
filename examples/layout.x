import std::io;

#[layout(AoS)]
struct Point {
    x: i32,
    y: i32,
}

#[layout(SoA)]
struct Sample {
    x: i32,
    y: i32,
}

fn main() -> i32 {
    let mut rows: [Point; 2] = [
        Point { x: 1, y: 10 },
        Point { x: 2, y: 20 },
    ];
    let mut columns: [Sample; 2] = [
        Sample { x: 1, y: 10 },
        Sample { x: 2, y: 20 },
    ];
    rows[1].x = rows[0].x + 3;
    columns[1].x = columns[0].x + 3;

    io::print("layout: AoS second x = ");
    io::println(rows[1].x);
    io::print("layout: SoA second x = ");
    io::println(columns[1].x);
    if rows[1].x != 4 || columns[1].x != 4
        || rows[1].y != 20 || columns[1].y != 20 {
        return 1;
    }

    io::println("layout: passed");
    return 0;
}
