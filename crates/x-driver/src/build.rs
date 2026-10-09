use super::*;
use x_compile::CompilationFailure;
use x_compile::CompileArtifact;
use x_compile::IrCompileOptions;
use x_compile::ObjectCompileOptions;
use x_diagnostics::DriverError;
use x_utils::artifacts::ArtifactPaths;

mod fast;

#[derive(Debug, Clone, Default)]
pub struct BuildOptions<'a> {
    pub emit_llvm: bool,
    pub show_parse: bool,
    pub libs: &'a [String],
    pub lib_paths: &'a [String],
    pub include_paths: &'a [String],
    pub optimisation_level: u8,
    pub save_ir: bool,
    pub target_triple: Option<&'a str>,
    pub emit_object: bool,
    pub freestanding: bool,
    pub link_arguments: &'a [String],
}

impl<'a> BuildOptions<'a> {
    fn object_options(&self) -> ObjectCompileOptions<'a> {
        let target_triple = self.target_triple.or_else(|| {
            self.freestanding
                .then_some(x_utils::target::ProductTarget::FreestandingX86.triple())
        });

        ObjectCompileOptions {
            save_ir: self.save_ir,
            ir: IrCompileOptions {
                show_parse: self.show_parse,
                discard_value_names: !self.emit_llvm && !self.save_ir,
                optimisation_level: x_utils::optimization::NativeOptimization::new(
                    self.optimisation_level,
                )
                .level(),
                target_triple,
                ..Default::default()
            },
        }
    }

    fn link_options<'build>(&'build self, target_triple: &'build str) -> LinkOptions<'build> {
        LinkOptions {
            libs: self.libs,
            lib_paths: self.lib_paths,
            include_paths: self.include_paths,
            optimisation_level: x_utils::optimization::NativeOptimization::new(
                self.optimisation_level,
            )
            .level(),
            target_triple: Some(target_triple),
            freestanding: self.freestanding,
            passthrough_arguments: self.link_arguments,
            reproducible: true,
            gc_sections: false,
        }
    }
}

fn prepare_object_path(output: &str) -> Result<PathBuf, CompilationFailure> {
    let path = ArtifactPaths::new(output).object();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| DriverError::project_object_directory(parent.display(), error))?;
    }

    Ok(path)
}

#[derive(Debug)]
pub struct BuildArtifact {
    pub path: PathBuf,
    pub compilation: CompileArtifact,
    pub executable: bool,
}

#[derive(Debug)]
pub enum BuildOutput {
    Native(BuildArtifact),
    C(CBuildArtifact),
}

impl BuildOutput {
    pub fn path(&self) -> &Path {
        match self {
            Self::Native(artifact) => &artifact.path,
            Self::C(artifact) => &artifact.path,
        }
    }

    pub fn executable(&self) -> bool {
        match self {
            Self::Native(artifact) => artifact.executable,
            Self::C(_) => true,
        }
    }
}

#[derive(Debug)]
pub struct CBuildArtifact {
    pub path: PathBuf,
    pub target_triple: String,
    pub compilation: x_compile::CArtifact,
    pub debug_artifacts: Vec<x_compile::DebugArtifact>,
    pub elapsed_seconds: f64,
}
