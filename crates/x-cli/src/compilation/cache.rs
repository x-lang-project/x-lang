use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{BuildOptions, CompilerInput};
use inputs::*;
use x_compile::{CompileArtifact, CompileMetrics};
use x_driver::manifest_project::resolve_manifest_modules;
use x_driver::{BuildArtifact, BuildOutput};
use x_project::ModuleResolver;
use x_project::manifest::{Manifest, lock::ensure_manifest_lock};
use x_utils::project::{LOCK_FILE_NAME, MANIFEST_FILE_NAME};
use x_utils::target::ProductTarget;

#[derive(Serialize, Deserialize)]
struct Entry {
    key: String,
    path: PathBuf,
    object_path: PathBuf,
    executable_digest: String,
    target: String,
    ir_bytes: usize,
    ir_lines: usize,
    dependencies: x_link::executable_cache::Dependencies,
}
pub(super) struct Request {
    key: String,
    path: PathBuf,
}
fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest(bytes: &[u8]) -> String {
    hex_digest(&Sha256::digest(bytes))
}

fn update(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

impl Request {
    pub(super) fn prepare(input: &CompilerInput, options: BuildOptions<'_>) -> Option<Self> {
        if options.no_cache
            || options.save_ir
            || options.emit_llvm
            || options.target.is_some_and(|type_value| {
                ProductTarget::parse(type_value) != Some(ProductTarget::WindowsMsvc)
            })
            || env::var_os("X_TCC_BACKEND").is_some()
            || env::var_os("X_TCC").is_some()
        {
            return None;
        }

        let mut hash = Sha256::new();
        update(&mut hash, b"x-executable-cache-v1");
        update(&mut hash, &x_link::executable_cache::toolchain_identity()?);
        update(
            &mut hash,
            format!("{:?}", env::vars_os().collect::<BTreeMap<_, _>>()).as_bytes(),
        );
        update(
            &mut hash,
            format!("{:?}", env::current_dir().ok()?).as_bytes(),
        );
        update(
            &mut hash,
            format!(
                "{:?}",
                (options.output, options.optimisation_level, options.target)
            )
            .as_bytes(),
        );
        match input {
            CompilerInput::Source(path) => {
                // Spelling affects default output names, especially symlink aliases.
                update(&mut hash, format!("source:{path:?}").as_bytes());
                hash_file(&mut hash, path)?;
            }
            CompilerInput::Project(path) => {
                update(&mut hash, format!("project:{path:?}").as_bytes());
                let mut seen = BTreeSet::new();
                let manifest = manifests(&mut hash, path, &mut seen, 0)?;
                workspace_files(&mut hash, manifest.path())?;
                let package = manifest.package()?;
                let mut resolver = ModuleResolver::new(&package.entry);
                resolve_manifest_modules(&manifest, &mut resolver).ok()?;
                let modules = resolver
                    .modules()
                    .values()
                    .map(|member| (&member.name, &member.path, &member.source))
                    .collect::<BTreeSet<_>>();

                for (name, path, source) in modules {
                    update(&mut hash, name.as_bytes());
                    update(
                        &mut hash,
                        format!("{:?}", path.canonicalize().ok()?).as_bytes(),
                    );
                    update(&mut hash, source.as_bytes());
                }
            }
        }

        let key = hex_digest(&hash.finalize());
        let root = env::var_os("X_EXECUTABLE_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| env::temp_dir().join("x-lang-executables-v1"));

        Some(Self {
            path: root.join(format!("{key}.json")),
            key,
        })
    }

    pub(super) fn load(&self) -> Option<BuildOutput> {
        let entry: Entry = serde_json::from_slice(&fs::read(&self.path).ok()?).ok()?;

        if entry.key != self.key
            || !x_link::executable_cache::validate(&entry.dependencies)
            || digest(&fs::read(&entry.path).ok()?) != entry.executable_digest
        {
            return None;
        }

        Some(BuildOutput::Native(BuildArtifact {
            path: entry.path,
            executable: true,
            compilation: CompileArtifact {
                path: entry.object_path,
                target_triple: entry.target,
                metrics: CompileMetrics {
                    elapsed_seconds: 0.0,
                    ir_bytes: entry.ir_bytes,
                    ir_lines: entry.ir_lines,
                },
                ir: String::new(),
                debug_artifacts: Vec::new(),
            },
        }))
    }

    pub(super) fn store(&self, artifact: &BuildOutput) -> Option<()> {
        let BuildOutput::Native(artifact) = artifact else {
            return None;
        };

        if !artifact.executable || !artifact.compilation.debug_artifacts.is_empty() {
            return None;
        }

        let dependencies = x_link::executable_cache::take_dependencies()?;

        if !x_link::executable_cache::validate(&dependencies) {
            return None;
        }

        let entry = Entry {
            key: self.key.clone(),
            path: artifact.path.clone(),
            object_path: artifact.compilation.path.clone(),
            executable_digest: digest(&fs::read(&artifact.path).ok()?),
            target: artifact.compilation.target_triple.clone(),
            ir_bytes: artifact.compilation.metrics.ir_bytes,
            ir_lines: artifact.compilation.metrics.ir_lines,
            dependencies,
        };
        let root = self.path.parent()?;
        fs::create_dir_all(root).ok()?;
        let mut temporary =
            x_utils::temporary::TemporaryFile::create(root, "executable-cache").ok()?;
        temporary
            .write_and_sync(&serde_json::to_vec(&entry).ok()?)
            .ok()?;
        // Publish complete metadata; a racing writer can only cause a miss.
        if !self.path.exists() {
            temporary.install(&self.path).ok()?;
        } else {
            fs::remove_file(&self.path).ok()?;
            temporary.install(&self.path).ok()?;
        }

        Some(())
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

mod inputs;
