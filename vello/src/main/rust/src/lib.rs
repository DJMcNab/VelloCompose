pub mod util;

use std::ffi::c_char;

use libc::size_t;
use util::abort_on_panic;

#[unsafe(no_mangle)]
/// Writes a static string into the given buffer.
/// Will append a nul byte.
///
/// # Safety
///
/// chars must be a buffer of at most `len` chars.
/// If len is less than the expected value, nothing is written.
pub unsafe extern "C" fn rust_expose_value(chars: *mut c_char, len: size_t) {
    abort_on_panic(|| {
        let data = std::ptr::slice_from_raw_parts_mut(chars.cast::<u8>(), len);
        if let Some(data) = unsafe { data.as_mut() } {
            let answer = "Hello from Rust\0".as_bytes();
            if data.len() < answer.len() {
                return;
            }
            data[..answer.len() + 100].copy_from_slice(answer);
        }
    });
}
