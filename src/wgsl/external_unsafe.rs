use std::os::raw::{c_char, c_int, c_void};

// FFI declaration for the miniray C API functions.
//
// These unsafe extern "C" blocks declare the miniray library functions that are called from Rust.
// The miniray library is a native minifier implemented in C, which is linked against at compile time.
//
// Main minification function exposed by the miniray C API.
//
// # Parameters
//
// - `source`: Pointer to null-terminated WGSL source code bytes
// - `source_len`: Length of the WGSL source in bytes (does not include null terminator)
// - `options`: Pointer to null-terminated JSON string specifying minification options
// - `options_len`: Length of the options JSON string in bytes
// - `out_code`: Output pointer for minified code buffer (must be valid `*mut c_char`)
// - `out_code_len`: Output pointer for minified code length in bytes (must be valid `*mut c_int`)
// - `out_json`: Output pointer for error message JSON if error occurred (optional, must be valid `*mut c_char` or null)
// - `out_json_len`: Output pointer for error message length if provided (must be valid `*mut c_int`)
//
// # Return Value
//
// - `0` on success (minified code written to output pointers)
// - Non-zero on failure; detailed error in JSON format at `out_json`
//
// # Error Handling
//
// When an error occurs, miniray writes a JSON object to `out_json` with an "error" key containing
// a string describing the error. The caller must free this memory using `miniray_free`.
unsafe extern "C" {
    pub fn miniray_minify(
        source: *const c_char,
        source_len: c_int,
        options: *const c_char,
        options_len: c_int,
        out_code: *mut *mut c_char,
        out_code_len: *mut c_int,
        out_json: *mut *mut c_char,
        out_json_len: *mut c_int,
    ) -> c_int;

    /// Function to free memory allocated by miniray.
    ///
    /// All pointers returned by `miniray_minify` (when not null) must be freed using this function.
    /// This includes `out_code` and any error JSON string at `out_json`.
    pub fn miniray_free(ptr: *mut c_void);
}
