use super::*;

const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x100;
const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x1000;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(path: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> c_int;
    fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
}

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

struct Module(*mut c_void);
impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.0);
        }
    }
}

impl Library {
    pub(super) fn open(directory: &Path) -> Result<Self, CompilationFailure> {
        let path = directory.join("libtcc.dll");
        let dll_identity = NativeToolchain::file_identity(&path)
            .ok_or_else(|| failure("TinyCC DLL has no stable filesystem identity"))?;
        let wide = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain([0])
            .collect::<Vec<_>>();
        let module = unsafe {
            LoadLibraryExW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            )
        };

        if module.is_null() {
            return Err(failure(std::io::Error::last_os_error()));
        }

        let module_guard = Module(module);

        if NativeToolchain::file_identity(&path).as_ref() != Some(&dll_identity) {
            return Err(failure("TinyCC DLL changed while it was being loaded"));
        }
        macro_rules! function {
            ($name:literal, $ty:ty) => {{
                let pointer =
                    unsafe { GetProcAddress(module, concat!($name, "\0").as_ptr().cast()) };
                if pointer.is_null() {
                    return Err(failure(concat!("missing libtcc export ", $name)));
                }
                unsafe { std::mem::transmute::<*mut c_void, $ty>(pointer) }
            }};
        }
        let library = Self {
            module,
            dll_identity,
            directory: c_path(directory)?,
            new: function!("tcc_new", New),
            delete: function!("tcc_delete", Delete),
            set_path: function!("tcc_set_lib_path", SetPath),
            set_options: function!("tcc_set_options", SetOptions),
            set_error: function!("tcc_set_error_func", SetError),
            output_type: function!("tcc_set_output_type", OutputType),
            compile_string: function!("tcc_compile_string", StringOperation),
            add_file: function!("tcc_add_file", StringOperation),
            output_file: function!("tcc_output_file", StringOperation),
        };
        std::mem::forget(module_guard);

        Ok(library)
    }

    pub(super) fn dll_identity(&self) -> &[u8] {
        &self.dll_identity
    }

    #[cfg(feature = "tinycc-session")]
    pub(super) fn use_resource_directory(
        &mut self,
        directory: &Path,
    ) -> Result<(), CompilationFailure> {
        self.directory = c_path(directory)?;

        Ok(())
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

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.module);
        }
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
    pub(super) fn without_standard_libraries(&mut self) -> Result<(), CompilationFailure> {
        self.result(unsafe { (self.library.set_options)(self.state, c"-nostdlib".as_ptr()) })
    }

    #[cfg(feature = "tinycc-session")]
    pub(super) fn without_standard_includes(&mut self) -> Result<(), CompilationFailure> {
        // Generated input needs no headers; disable environment-dependent include searches.
        self.result(unsafe { (self.library.set_options)(self.state, c"-nostdinc".as_ptr()) })
    }

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

    pub(super) fn compile(&mut self, source: &str) -> Result<(), CompilationFailure> {
        let source = CString::new(source).map_err(failure)?;

        self.result(unsafe { (self.library.compile_string)(self.state, source.as_ptr()) })
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

pub(super) fn system_crt() -> Result<PathBuf, CompilationFailure> {
    let mut buffer = vec![0u16; 32768];
    let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };

    if length == 0 || length as usize >= buffer.len() {
        return Err(failure("cannot resolve the Windows system directory"));
    }

    let directory = String::from_utf16(
        buffer
            .get(..length as usize)
            .ok_or_else(|| failure("Windows system directory length is invalid"))?,
    )
    .map_err(failure)?;

    PathBuf::from(directory)
        .join("ucrtbase.dll")
        .canonicalize()
        .map_err(failure)
}

impl Drop for State<'_> {
    fn drop(&mut self) {
        // The boxed callback buffer and Library still live during tcc_delete.
        unsafe {
            (self.library.delete)(self.state);
        }
    }
}
