use std::env;

#[path = "../x-utils/build/llvm_policy.rs"]
mod llvm_policy;
#[path = "../x-utils/build/llvm_runtime.rs"]
mod llvm_runtime;
#[path = "../x-utils/build/llvm_toolchain.rs"]
mod llvm_toolchain;

fn main() {
    let target = env::var("TARGET").expect("Cargo must define TARGET");
    println!("cargo:rustc-env=X_HOST_TARGET={target}");
    llvm_runtime::stage_windows_llvm_runtime_next_to_binary();
    delay_windows_llvm_loading();
}

fn delay_windows_llvm_loading() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Delay LLVM loading until native generation so checking and CLI help work without it.
        println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-link-arg-bin=x=/DELAYLOAD:LLVM-C.dll");
    }
}
