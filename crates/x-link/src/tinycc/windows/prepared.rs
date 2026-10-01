use super::*;

/// Hold the DLL and package guards inside TinyCC's lock until every state is dropped.
pub(super) struct PreparedRuntime {
    // Fields drop in declaration order; release package guards and DLL before the lock.
    runtime: cache::RuntimeObject,
    pub(super) library: Library,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl PreparedRuntime {
    pub(super) fn new(config: &Configuration) -> Result<Self, CompilationFailure> {
        let guard = TINYCC
            .lock()
            .map_err(|_| failure("TinyCC serialisation lock poisoned"))?;
        let library = Library::open(&config.directory)?;
        let runtime = cache::runtime_object(&library, config)?;

        Ok(Self {
            library,
            runtime,
            _guard: guard,
        })
    }

    pub(super) fn executable(&self) -> Result<library::State<'_>, CompilationFailure> {
        executable_state(&self.library, &self.runtime.path)
    }
}

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
