use std::env;

#[path = "../x-utils/build/llvm_policy.rs"]
mod llvm_policy;
#[path = "../x-utils/build/llvm_toolchain.rs"]
mod llvm_toolchain;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let prefix = llvm_toolchain::llvm_prefix().expect("the pinned LLVM toolchain is required");

        println!(
            "cargo:rustc-link-search=native={}",
            prefix.join("lib").display()
        );
        println!("cargo:rustc-link-lib=dylib=LLVM-C");
    }
}
