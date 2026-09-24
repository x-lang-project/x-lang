use super::*;

pub(super) fn executable_state<'a>(
    library: &'a Library,
    runtime: &Path,
) -> Result<library::State<'a>, CompilationFailure> {
    let mut state = library.state()?;
    state.output_type(OutputKind::Executable)?;
    state.without_standard_libraries()?;
    state.add_file(runtime)?;

    Ok(state)
}

pub(super) fn attach_support(
    state: &mut library::State<'_>,
    directory: &Path,
    ucrt: &Path,
) -> Result<(), CompilationFailure> {
    state.add_file(&directory.join("lib/libtcc1.a"))?;
    state.add_file(&directory.join("lib/kernel32.def"))?;

    state.add_file(ucrt)
}
