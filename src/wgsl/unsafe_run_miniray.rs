use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_void},
    ptr,
};

use crate::wgsl::external_unsafe::{miniray_free, miniray_minify};

/// Runs the miniray minifier on WGSL source data with the given options.
///
/// # Arguments
///
/// - `data`: Raw bytes of WGSL source code to minify
/// - `options_json`: JSON string of minification options (as produced by [`build_options_json`])
///
/// # Returns
///
/// A `Vec<u8>` containing the minified WGSL shader code. The memory is automatically freed
/// by this function using `miniray_free`.
///
/// # Panics
///
/// This function panics in two cases:
/// 1. If miniray returns a non-zero error code
/// 2. If the C API fails to allocate output buffers (`out_code` is null)
///
/// The panic message includes the JSON error object from miniray (if provided) or a generic error code string.
pub fn run_miniray(data: &[u8], options_json: &str) -> Vec<u8> {
    // Create C string from options JSON for FFI call
    let opts_cstr = CString::new(options_json).unwrap();

    // Output pointers initialized to null (C API will set these if successful)
    let mut out_code: *mut c_char = ptr::null_mut();
    let mut out_code_len: c_int = 0;

    // Error message pointer and length (may be null if no error)
    let mut out_json: *mut c_char = ptr::null_mut();
    let mut out_json_len: c_int = 0;

    // Call miniray_minify through FFI
    let rc = unsafe {
        miniray_minify(
            // Input WGSL source data as C string pointer
            data.as_ptr() as *const c_char,
            // Source length in bytes (cast from usize to signed int)
            data.len() as c_int,
            // Options JSON string pointer
            opts_cstr.as_ptr(),
            // Options JSON length in bytes (not including null terminator for FFI)
            opts_cstr.as_bytes().len() as c_int,
            // Output pointers for result buffers
            &mut out_code,
            &mut out_code_len,
            &mut out_json,
            &mut out_json_len,
        )
    };

    // Panic if miniray returns an error code or fails to allocate output buffer.
    // This prevents invalid shader code from being compiled into the binary.
    if rc != 0 || out_code.is_null() {
        // If error JSON was provided, parse and use that message; otherwise use generic error code
        let msg = if out_json.is_null() {
            format!("miniray_minify failed: {}", rc)
        } else {
            let s = unsafe { CStr::from_ptr(out_json).to_string_lossy().into_owned() };
            // Free the error message buffer allocated by miniray
            unsafe { miniray_free(out_json as *mut c_void) };
            s
        };

        panic!("miniray minification failed: {}", msg);
    }

    // Convert the output buffer to a Rust Vec<u8>
    let result =
        unsafe { std::slice::from_raw_parts(out_code as *const u8, out_code_len as usize) }
            .to_vec();

    // Free all memory allocated by miniray (output code and optional error JSON)
    unsafe {
        miniray_free(out_code as *mut c_void);
        if !out_json.is_null() {
            miniray_free(out_json as *mut c_void);
        }
    }

    // Return the minified WGSL code to the caller
    result
}
