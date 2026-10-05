use super::SourceContext;
use x_ast::{Program, SourceError};
use x_borrow_checker::BorrowChecker;
use x_diagnostics::CompilationFailure;
use x_typechecker::TypeChecker;

pub(super) fn typecheck(
    program: Program,
    context: &mut SourceContext<'_>,
) -> Result<Program, CompilationFailure> {
    let mut checker = TypeChecker::new();

    if let Some(sources) = context.catalog.as_deref() {
        checker.set_declaration_sources(&sources.declarations);
    }

    let checked = checker.check_and_take_program(program);

    if let Some(sources) = context.catalog.as_deref_mut() {
        sources
            .declarations
            .clone_from(checker.declaration_sources());
    }

    checked.map_err(|error| {
        let (path, source) = context.source_for(error.source_span());

        error.into_failure(source, path)
    })
}

fn borrow_check_with_sources(
    program: &Program,
    no_borrow_check: bool,
    context: &SourceContext<'_>,
) -> Result<(), CompilationFailure> {
    if no_borrow_check {
        return Ok(());
    }

    let mut borrow_checker = BorrowChecker::new();
    let checked = match context.catalog.as_deref() {
        Some(sources) => {
            borrow_checker.check_with_declaration_sources(program, &sources.declarations)
        }
        None => borrow_checker.check(program),
    };

    checked.map_err(|error| {
        context.diagnostic_failure(error.source_span(), |span| error.into_diagnostic(span))
    })
}
