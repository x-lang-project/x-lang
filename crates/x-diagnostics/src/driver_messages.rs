use std::{fmt, path::Path};

use crate::Error;
use crate::{CompilationFailure, OperationalDiagnostic};

#[derive(Debug, Clone, Copy)]
pub enum OsBuildStage<'a> {
    Compile(&'a Path),
    AssembleMultiboot,
    LinkKernel,
    CreateIso,
}

impl fmt::Display for OsBuildStage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(path) => write!(formatter, "compiling {}", path.display()),
            Self::AssembleMultiboot => formatter.write_str("assembling the multiboot header"),
            Self::LinkKernel => formatter.write_str("linking the freestanding kernel"),
            Self::CreateIso => formatter.write_str("creating the bootable ISO"),
        }
    }
}

pub struct DriverError;

impl DriverError {
    pub fn iso_tool_unavailable(path: impl fmt::Display) -> String {
        Error::new(format!("warning: grub-mkrescue is unavailable; kernel remains directly bootable by QEMU: {path}")).render_detail()
    }
}

mod os;
