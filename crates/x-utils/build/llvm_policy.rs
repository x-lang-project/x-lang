pub const LLVM_VERSION: &str = include_str!("llvm-version.txt").trim_ascii();
pub const LLVM_PREFIX_ENV: &str = "LLVM_SYS_221_PREFIX";

pub fn clang_is_pinned(version: &str) -> bool {
    let mut words = version
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();

    while let Some(word) = words.next() {
        if word == "version" {
            return words.next() == Some(LLVM_VERSION);
        }
    }

    false
}
