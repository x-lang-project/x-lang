<p align="center">
  <img width="150" src="https://avatars.githubusercontent.com/u/275293734?s=200&v=4" alt="X">
</p>

<h1 align="center">X</h1>

<p align="center">A systems programming language with checked semantic contracts.</p>

```x
memoise pure fn square(value: i32) -> i32 {
    return value * value;
}

fn main() -> i32 {
    return square(7) - 49;
}
```

## Getting started

Requires Rust 1.97.1 and LLVM/Clang 22.1.8.

```sh
cargo run --release --locked -p x-cli -- new hello
cargo run --release --locked -p x-cli -- run hello
```
