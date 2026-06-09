#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86,
    X86_64,
    RiscV32,
    RiscV64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetSpec<'a> {
    pub triple: &'a str,
    pub architecture: Architecture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductTarget {
    WindowsMsvc,
    LinuxGnu,
    FreestandingX86,
}

impl ProductTarget {
    pub const fn triple(self) -> &'static str {
        match self {
            Self::WindowsMsvc => "x86_64-pc-windows-msvc",
            Self::LinuxGnu => "x86_64-unknown-linux-gnu",
            Self::FreestandingX86 => "i386-unknown-none-elf",
        }
    }

    pub fn parse(triple: &str) -> Option<Self> {
        [Self::WindowsMsvc, Self::LinuxGnu, Self::FreestandingX86]
            .into_iter()
            .find(|target| target.triple() == triple)
    }

    pub const fn is_hosted(self) -> bool {
        matches!(self, Self::WindowsMsvc | Self::LinuxGnu)
    }
}

pub struct CheriPurecap;

/// Windows calling conventions with ELF objects for the optional TinyCC linker.
pub struct TinyCcWindows;

impl TinyCcWindows {
    pub const HOST_TRIPLE: &str = ProductTarget::WindowsMsvc.triple();
    pub const TRIPLE: &str = "x86_64-pc-windows-elf";
    pub const STACK_PROBE: &str = "x_llvm_chkstk";
}

impl CheriPurecap {
    pub const POINTER_IR_TYPE: &str = "ptr addrspace(200)";
    pub const PROFILE: &str = "cheri-riscv64-purecap";
    pub const TRIPLE: &str = "riscv64-unknown-freebsd";
    pub const ARCH: &str = "rv64gcxcheri";
    pub const ABI: &str = "l64pc128d";
    pub const DATA_LAYOUT: &str =
        "e-m:e-p200:128:128-i64:64-i128:128-n32:64-S128-A200-P200-G200-ni:200";
}

impl<'a> TargetSpec<'a> {
    pub fn compiler_arguments(self) -> Vec<String> {
        let mut arguments = vec![format!("--target={}", self.triple)];

        if self.architecture == Architecture::X86 {
            arguments.push("-m32".into());
        }

        arguments
    }
}
