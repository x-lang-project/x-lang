use std::os::windows::fs::MetadataExt;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::*;

mod identity;

static NEXT_OBJECT: AtomicU64 = AtomicU64::new(0);
const CORE: &str = x_runtime::source::HOSTED;
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

fn package_files(directory: &Path) -> Result<Vec<PathBuf>, CompilationFailure> {
    let mut pending = ["include", "lib", "libtcc"]
        .map(|name| directory.join(name))
        .to_vec();
    let mut files = vec![directory.join("libtcc.dll")];

    while let Some(path) = pending.pop() {
        let metadata = path.symlink_metadata().map_err(failure)?;

        if metadata.file_attributes() & 0x400 != 0 {
            return Err(failure("TinyCC cache package contains a symbolic link"));
        }

        if metadata.is_dir() {
            for entry in fs::read_dir(path).map_err(failure)? {
                pending.push(entry.map_err(failure)?.path());
            }
        } else if metadata.is_file() {
            files.push(path);
        }
    }

    files.sort();

    Ok(files)
}

fn update_part(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
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

fn cache_key_with_signature(
    config: &Configuration,
    source: &str,
    guarded: bool,
    captured: Option<&[u8]>,
) -> Result<String, CompilationFailure> {
    let mut hash = Sha256::new();
    update_part(&mut hash, b"tinycc-win64-ucrt-core-v3;-nostdlib;elf64");
    update_part(&mut hash, source.as_bytes());
    update_part(
        &mut hash,
        format!("{:?}:{}", config.directory, config.object_type.abi()).as_bytes(),
    );
    update_part(&mut hash, &file_signature(&config.ucrt)?);
    update_part(
        &mut hash,
        &file_signature(&config.directory.join("libtcc.dll"))?,
    );
    // Reuse this capture once; later validations must enumerate the namespace again.
    let signature = if guarded {
        captured.map(std::borrow::Cow::Borrowed).or_else(|| {
            identity::package_signature(&config.directory)
                .ok()
                .map(std::borrow::Cow::Owned)
        })
    } else {
        None
    };

    if let Some(signature) = signature {
        update_part(&mut hash, b"directory-identity");
        update_part(&mut hash, &signature);
    } else {
        // Without directory identities, validate complete file contents.
        update_part(&mut hash, b"package-content");

        for file in package_files(&config.directory)? {
            update_part(
                &mut hash,
                file.strip_prefix(&config.directory)
                    .map_err(failure)?
                    .as_os_str()
                    .as_encoded_bytes(),
            );
            update_part(&mut hash, &fs::read(&file).map_err(failure)?);
        }
    }

    Ok(hex(&hash.finalize()))
}

fn cache_key(
    config: &Configuration,
    source: &str,
    guarded: bool,
) -> Result<String, CompilationFailure> {
    cache_key_with_signature(config, source, guarded, None)
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

pub(super) fn runtime_object(
    library: &Library,
    config: &Configuration,
) -> Result<RuntimeObject, CompilationFailure> {
    let probe = PROBE.replace("__X_PROBE__", x_utils::target::TinyCcWindows::STACK_PROBE);
    let source = format!("{UCRT}\n{probe}\n{CORE}");
    validate_loaded_package(library, config)?;
    let (package_guards, captured) = identity::guard_package(&config.directory)
        .ok()
        .map(|(guards, signature)| (Some(guards), Some(signature)))
        .unwrap_or_default();
    let guarded = package_guards.is_some();
    let key = cache_key_with_signature(config, &source, guarded, captured.as_deref())?;
    let directory = env::var_os("X_RUNTIME_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("x-tinycc-runtime-v3")
        .join(&key);
    let path = directory.join("runtime.o");
    let digest_path = directory.join("runtime.sha256");

    if verified_object(&path, &digest_path) && cache_key(config, &source, guarded)? == key {
        validate_loaded_package(library, config)?;

        return Ok(RuntimeObject {
            path,
            temporary: false,
            _package_guards: package_guards,
        });
    }

    fs::create_dir_all(&directory).map_err(failure)?;
    let sequence = NEXT_OBJECT.fetch_add(1, Ordering::Relaxed);
    let temporary = directory.join(format!("runtime-{}-{sequence}.o", std::process::id()));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(failure)?;
    let mut object = RuntimeObject {
        path: temporary,
        temporary: true,
        _package_guards: package_guards,
    };
    let mut state = library.state()?;
    state.output_type(config.object_type)?;
    state.without_standard_libraries()?;
    state.compile(&source)?;
    state.output_file(&object.path)?;
    let bytes = fs::read(&object.path).map_err(failure)?;

    if !elf_object(&bytes) {
        return Err(failure(
            "libtcc object output does not match its package header ABI",
        ));
    }

    validate_loaded_package(library, config)?;
    if cache_key(config, &source, guarded)? != key {
        return Err(failure(
            "TinyCC package changed while compiling its runtime",
        ));
    }
    // Keep a private object if publication races; existing cache entries are immutable.
    if fs::hard_link(&object.path, &path).is_ok() {
        let _ = fs::write(&digest_path, hex(&Sha256::digest(bytes)));
        let _ = fs::remove_file(&object.path);
        object.path = path;
        object.temporary = false;
    }

    Ok(object)
}

mod object_format;
pub(crate) use object_format::*;
