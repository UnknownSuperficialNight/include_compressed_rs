#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

//! # include_compressed
//!
//! A collection of procedural macros for embedding compressed files at compile time.
//!
//! ## General Compression
//! These macros can be used to embed any file into your binary in a compressed format:
//! - [`brotli_compress!`] - Compresses a file using Brotli and returns a static byte slice.
//! - [`zstd_compress!`] - Compresses a file using Zstandard and returns a static byte slice.
//!
//! ## WGSL Minification
//! **Requires the Go compiler to be installed.**
//!
//! When the `wgsl_minify` feature is enabled, WGSL shaders are minified before
//! compression to reduce their size:
//! - [`include_minified_wgsl!`] - Minifies a WGSL shader and returns it as a string literal.
//! - [`include_minified_wgsl_compressed!`] - Minifies a WGSL shader, then compresses it,
//!   returning the result as a static byte array.
//!
//! ## Examples
//!
//! ```Rust
//! // Simple Brotli compression of any file
//! let compressed = brotli_compress!("assets/data.bin");
//!
//! // Simple Zstd compression of a shader
//! let compressed = zstd_compress!("shaders/main.wgsl");
//!
//! // Minify and compress with specific settings (requires "wgsl_minify" feature)
//! #[cfg(feature = "wgsl_minify")]
//! let compressed = include_minified_wgsl_compressed!(
//!     "shaders/main.wgsl",
//!     codec = "zstd",
//!     quality = 4
//! );
//! ```

extern crate proc_macro;

mod brotli;
mod parsing;
mod zstd;

#[cfg(feature = "wgsl_minify")]
mod wgsl;

use proc_macro::TokenStream;

/// Embeds a file into the binary compressed with Brotli.
///
/// This macro reads the specified file during compilation, compresses it using
/// Brotli, and returns a static byte slice that becomes part of your source code.
///
/// # Examples
///
/// ```Rust
/// let compressed_data: &'static [u8] = brotli_compress!("shader.wgsl");
/// ```
#[proc_macro]
pub fn brotli_compress(input: TokenStream) -> TokenStream {
    brotli::brotli_compress_impl(input)
}

/// Embeds a file into the binary compressed with Zstd.
///
/// This macro reads the specified file during compilation, compresses it using
/// Zstandard (Zstd), and returns a static byte slice that becomes part of your source code.
///
/// # Examples
///
/// ```Rust
/// let compressed_data: &'static [u8] = zstd_compress!("shader.wgsl");
/// ```
#[proc_macro]
pub fn zstd_compress(input: TokenStream) -> TokenStream {
    zstd::zstd_compress_impl(input)
}

/// Embeds a minified WGSL shader as a string literal into the binary.
///
/// This macro invokes the miniray tool to strip comments, collapse whitespace,
/// and optimize WGSL syntax, then returns the result as a string literal
/// embedded in your Rust code. The actual WGSL source (not byte arrays) becomes
/// part of the compiled artifact.
///
/// **Note:** This macro requires the `wgsl_minify` feature to be enabled in `Cargo.toml`
/// and the Go compiler to be installed and accessible.
///
/// # Examples
///
/// ```Rust
/// // Default: Minify only
/// let code = include_minified_wgsl!("shader.wgsl");
///
/// // Minify another WGSL file
/// let code = include_minified_wgsl!("examples/simple_shader.wgsl");
/// ```
#[cfg(feature = "wgsl_minify")]
#[proc_macro]
pub fn include_minified_wgsl(input: TokenStream) -> TokenStream {
    wgsl::include_minified_wgsl_impl(input)
}

/// Embeds a minified and compressed WGSL shader as a byte slice literal into the binary.
///
/// This macro combines two steps: first invoking the miniray tool to optimize
/// the WGSL syntax, then compressing the result using Brotli or Zstandard.
/// The final output is a static byte array that becomes part of your Rust source
/// code as literal values.
///
/// **Note:** This macro requires the `wgsl_minify` feature to be enabled in `Cargo.toml`
/// and the Go compiler to be installed and accessible.
///
/// ### Arguments
/// - `path`: Required. A string literal containing the path to the `.wgsl` file.
/// - `codec`: Optional. Compression algorithm: `"brotli"` (default) or `"zstd"`.
/// - `quality`: Optional. Integer compression level for the selected codec.
/// - `options`: Optional. Additional miniray configuration as JSON string.
///
/// # Examples
///
/// ```Rust
/// // Default: Minify + Brotli compression
/// let data = include_minified_wgsl_compressed!("shader.wgsl");
///
/// // Minify + Zstd with quality level 5
/// let data = include_minified_wgsl_compressed!(
///     "shader.wgsl",
///     codec = "zstd",
///     quality = 5
/// );
/// ```
#[cfg(feature = "wgsl_minify")]
#[proc_macro]
pub fn include_minified_wgsl_compressed(input: TokenStream) -> TokenStream {
    wgsl::include_minified_wgsl_compressed_impl(input)
}
