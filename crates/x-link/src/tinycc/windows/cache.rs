use std::os::windows::fs::MetadataExt;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::*;

mod identity;

const PROBE: &str = r#"__asm__(".text\n.globl __X_PROBE__\n__X_PROBE__:\npush %rcx\npush %rax\nlea 24(%rsp),%rcx\ncmp $4096,%rax\njb 2f\n1:\nsub $4096,%rcx\ntestb $0,(%rcx)\nsub $4096,%rax\ncmp $4096,%rax\njae 1b\n2:\nsub %rax,%rcx\ntestb $0,(%rcx)\npop %rax\npop %rcx\nret\n");"#;

pub(super) struct RuntimeObject {
    pub(super) path: PathBuf,
    temporary: bool,
    _package_guards: Option<Vec<fs::File>>,
}

#[cfg(feature = "tinycc-session")]
impl RuntimeObject {
    pub(super) fn is_guarded(&self) -> bool {
        self._package_guards.is_some()
    }
}

impl Drop for RuntimeObject {
    fn drop(&mut self) {
        if self.temporary {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn file_signature(path: &Path) -> Result<Vec<u8>, CompilationFailure> {
    if let Some(identity) = NativeToolchain::file_identity(path) {
        return Ok(identity);
    }
    // Filesystems without stable file/change identities use content validation.
    fs::read(path)
        .map(|bytes| Sha256::digest(bytes).to_vec())
        .map_err(failure)
}

fn validate_loaded_package(
    library: &Library,
    config: &Configuration,
) -> Result<(), CompilationFailure> {
    if NativeToolchain::file_identity(&config.directory.join("libtcc.dll")).as_deref()
        != Some(library.dll_identity())
    {
        return Err(failure("TinyCC DLL changed after it was loaded"));
    }

    let header = fs::read_to_string(config.directory.join("libtcc/libtcc.h")).map_err(failure)?;
    validate_options_abi(&header)?;
    let current = header_object_output_type(&header);

    if current != Some(config.object_type.abi()) {
        return Err(failure(
            "TinyCC package header ABI changed during configuration",
        ));
    }

    Ok(())
}

fn verified_object(path: &Path, digest_path: &Path) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };

    elf_object(&bytes)
        && fs::read_to_string(digest_path)
            .is_ok_and(|expected| expected == hex(&Sha256::digest(bytes)))
}

mod object_format;
pub(crate) use object_format::*;
