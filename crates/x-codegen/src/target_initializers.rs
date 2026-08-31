use std::ffi::c_int;

use inkwell::llvm_sys::target;

// LLVM-C.dll does not export the header-only target initialisers used by Inkwell.
#[unsafe(export_name = "LLVM_InitializeAllTargetInfos")]
extern "C" fn initialize_all_target_infos() {
    unsafe {
        target::LLVMInitializeX86TargetInfo();
        target::LLVMInitializeRISCVTargetInfo();
    }
}

#[unsafe(export_name = "LLVM_InitializeAllTargets")]
extern "C" fn initialize_all_targets() {
    unsafe {
        target::LLVMInitializeX86Target();
        target::LLVMInitializeRISCVTarget();
    }
}

#[unsafe(export_name = "LLVM_InitializeAllTargetMCs")]
extern "C" fn initialize_all_target_mcs() {
    unsafe {
        target::LLVMInitializeX86TargetMC();
        target::LLVMInitializeRISCVTargetMC();
    }
}

#[unsafe(export_name = "LLVM_InitializeAllAsmPrinters")]
extern "C" fn initialize_all_asm_printers() {
    unsafe {
        target::LLVMInitializeX86AsmPrinter();
        target::LLVMInitializeRISCVAsmPrinter();
    }
}

#[unsafe(export_name = "LLVM_InitializeAllAsmParsers")]
extern "C" fn initialize_all_asm_parsers() {
    unsafe {
        target::LLVMInitializeX86AsmParser();
        target::LLVMInitializeRISCVAsmParser();
    }
}

#[unsafe(export_name = "LLVM_InitializeAllDisassemblers")]
extern "C" fn initialize_all_disassemblers() {
    unsafe {
        target::LLVMInitializeX86Disassembler();
        target::LLVMInitializeRISCVDisassembler();
    }
}

fn initialize_native(initialisers: &[unsafe extern "C" fn()]) -> c_int {
    if !cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        return 1;
    }

    for initialise in initialisers {
        unsafe { initialise() };
    }

    0
}

#[unsafe(export_name = "LLVM_InitializeNativeTarget")]
extern "C" fn initialize_native_target() -> c_int {
    initialize_native(&[
        target::LLVMInitializeX86TargetInfo,
        target::LLVMInitializeX86Target,
        target::LLVMInitializeX86TargetMC,
    ])
}

#[unsafe(export_name = "LLVM_InitializeNativeAsmPrinter")]
extern "C" fn initialize_native_asm_printer() -> c_int {
    initialize_native(&[target::LLVMInitializeX86AsmPrinter])
}

#[unsafe(export_name = "LLVM_InitializeNativeAsmParser")]
extern "C" fn initialize_native_asm_parser() -> c_int {
    initialize_native(&[target::LLVMInitializeX86AsmParser])
}

#[unsafe(export_name = "LLVM_InitializeNativeDisassembler")]
extern "C" fn initialize_native_disassembler() -> c_int {
    initialize_native(&[target::LLVMInitializeX86Disassembler])
}
