#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

mod artifact;
mod c_artifact;
#[cfg(feature = "c-backend")]
pub mod c_backend;
mod cheri_subset;
mod executable_contracts;
mod frontend;
pub mod jit;
mod mir_ir;
mod mir_object;
mod module_names;
mod module_pipeline;
mod object;
mod options;
mod source_ir;
mod source_mir;
mod source_object;
mod sources;

pub use artifact::{CompileArtifact, CompileMetrics, DebugArtifact, GeneratedIr};
pub use c_artifact::{CArtifact, CEmissionMetrics};
pub use executable_contracts::validate_executable_contracts;
pub use mir_object::MirObjectOptions;
pub use module_names::qualify_module;
pub use module_pipeline::{
    ModuleCompileOptions, compile_module_graph_to_mir, compile_owned_module_graph_to_mir,
};
pub use options::{IrCompileOptions, ObjectCompileOptions};
pub use source_ir::{
    compile_and_generate_ir, compile_source_to_cheri_riscv64_ir, compile_source_to_ir,
};
pub use source_mir::{SourceMirArtifact, SourceMirOptions, check_source, compile_source_to_mir};

pub use sources::SourceCatalog;
pub use x_diagnostics::CompilationFailure;

use object::object_context;
use source_ir::compile_and_generate_module;

pub const CHERI_RISCV64_PURECAP_PROFILE: &str = x_utils::target::CheriPurecap::PROFILE;
pub const CHERI_RISCV64_TRIPLE: &str = x_utils::target::CheriPurecap::TRIPLE;
pub const CHERI_RISCV64_DATALAYOUT: &str = x_utils::target::CheriPurecap::DATA_LAYOUT;
