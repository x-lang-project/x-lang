use super::SourceContext;
use crate::SourceCatalog;
use x_ast::{Program, SourceError};
use x_diagnostics::{CompilationFailure, InternalDiagnostic, StageDiagnostic};
use x_hir::HirProgram;
use x_mir::MirProgram;

pub(super) fn lower_hir(
    program: Program,
    context: &SourceContext<'_>,
) -> Result<HirProgram, CompilationFailure> {
    let empty_sources = SourceCatalog::default();
    let sources = context.catalog.as_deref().unwrap_or(&empty_sources);
    let hir = x_hir::lower_program_with_sources(program, sources.declaration_spans()).map_err(
        |error| {
            let (path, source) = context.source_for(error.source_span());

            CompilationFailure::stage(source, path, StageDiagnostic::HIR_LOWERING, error)
        },
    )?;

    hir.validate()
        .map_err(|error| InternalDiagnostic::InvalidHir.into_failure(error))?;

    Ok(hir)
}

pub(super) fn lower_mir(
    hir: HirProgram,
    context: &SourceContext<'_>,
) -> Result<MirProgram, CompilationFailure> {
    let mir = x_mir::lower_program(hir).map_err(|error| {
        context.diagnostic_failure(error.source_span(), |span| error.into_diagnostic(span))
    })?;

    mir.validate()
        .map_err(|error| InternalDiagnostic::InvalidMir.into_failure(error))?;

    Ok(mir)
}
