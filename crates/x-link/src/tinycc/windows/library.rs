use super::*;

type New = unsafe extern "C" fn() -> *mut c_void;
type Delete = unsafe extern "C" fn(*mut c_void);
type SetPath = unsafe extern "C" fn(*mut c_void, *const c_char);
type SetOptions = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type SetError = unsafe extern "C" fn(
    *mut c_void,
    *mut c_void,
    unsafe extern "C" fn(*mut c_void, *const c_char),
);
type OutputType = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type StringOperation = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;

// TinyCC retains this address across calls, so keep the callback buffer boxed.
struct ErrorBuffer(Vec<String>);

pub(super) struct Library {
    module: *mut c_void,
    dll_identity: Vec<u8>,
    directory: CString,
    new: New,
    delete: Delete,
    set_path: SetPath,
    set_options: SetOptions,
    set_error: SetError,
    output_type: OutputType,
    compile_string: StringOperation,
    add_file: StringOperation,
    output_file: StringOperation,
}

impl Library {
    pub(super) fn dll_identity(&self) -> &[u8] {
        &self.dll_identity
    }

    pub(super) fn state(&self) -> Result<State<'_>, CompilationFailure> {
        let state = unsafe { (self.new)() };

        if state.is_null() {
            return Err(failure("tcc_new returned NULL"));
        }

        let mut errors = Box::new(ErrorBuffer(Vec::new()));
        unsafe {
            (self.set_path)(state, self.directory.as_ptr());
            (self.set_error)(
                state,
                (&mut *errors as *mut ErrorBuffer).cast(),
                error_callback,
            );
        }

        Ok(State {
            library: self,
            state,
            errors,
        })
    }
}

unsafe extern "C" fn error_callback(opaque: *mut c_void, message: *const c_char) {
    if opaque.is_null() || message.is_null() {
        return;
    }
    // No Rust panic may unwind through TinyCC's C frames.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let message = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned();
        unsafe { &mut *opaque.cast::<ErrorBuffer>() }
            .0
            .push(message);
    }));
}

pub(super) struct State<'a> {
    library: &'a Library,
    state: *mut c_void,
    errors: Box<ErrorBuffer>,
}

impl State<'_> {
    fn result(&self, code: c_int) -> Result<(), CompilationFailure> {
        if code < 0 {
            Err(failure(self.errors.0.join("\n")))
        } else {
            Ok(())
        }
    }

    pub(super) fn output_type(&mut self, kind: OutputKind) -> Result<(), CompilationFailure> {
        self.result(unsafe { (self.library.output_type)(self.state, kind.abi()) })
    }

    pub(super) fn add_file(&mut self, path: &Path) -> Result<(), CompilationFailure> {
        let path = c_path(path)?;

        self.result(unsafe { (self.library.add_file)(self.state, path.as_ptr()) })
    }

    pub(super) fn output_file(&mut self, path: &Path) -> Result<(), CompilationFailure> {
        let path = c_path(path)?;

        self.result(unsafe { (self.library.output_file)(self.state, path.as_ptr()) })
    }
}
