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
