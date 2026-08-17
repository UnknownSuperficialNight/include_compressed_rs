//! # include_compressed
//!
//! A collection of procedural macros for embedding compressed files at compile time.
//!
//! ## General Compression
//! These macros can be used to embed any file into your binary in a compressed format:
//! - [`brotli_compress!`] - Embeds a file compressed with Brotli.
//! - [`zstd_compress!`] - Embeds a file compressed with Zstd.
//!
//! ## WGSL Minification
//! **Requires the Go compiler to be installed.**
//!
//! When the `wgsl_minify` feature is enabled, WGSL shaders are minified before
//! compression to reduce their size:
//! - [`include_minified_wgsl!`] - Minifies a WGSL shader and embeds the raw bytes.
//! - [`include_minified_wgsl_compressed!`] - Minifies a WGSL shader and then
//!   compresses it using a specified codec.
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
/// This macro reads the file at the specified path during compilation and
/// compresses it. The resulting `&'static [u8]` is embedded directly into the executable.
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
/// This macro reads the file at the specified path during compilation and
/// compresses it. The resulting `&'static [u8]` is embedded directly into the executable.
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

/// Embeds a minified WGSL shader source into the binary.
///
/// This macro reads the WGSL file at the given path, strips comments and
/// unnecessary whitespace, and embeds the resulting bytes as a `&'static [u8]`.
///
/// **Note:** This macro requires the `wgsl_minify` feature to be enabled in `Cargo.toml`.
///
/// # Examples
///
/// ```Rust
/// // Default: Minify only
/// let data = include_minified_wgsl!("shader.wgsl");
///
/// // Minify another WGSL file
/// let data = include_minified_wgsl!("examples/simple_shader.wgsl");
/// ```
#[cfg(feature = "wgsl_minify")]
#[proc_macro]
pub fn include_minified_wgsl(input: TokenStream) -> TokenStream {
    wgsl::include_minified_wgsl_impl(input)
}

/// Embeds a minified and compressed WGSL shader source into the binary.
///
/// This macro combines minification and compression into a single step. It is
/// the most aggressive way to reduce the footprint of shaders in your binary.
///
/// **Note:** This macro requires the `wgsl_minify` feature to be enabled in `Cargo.toml`.
///
/// ### Arguments
/// - `path`: The path to the `.wgsl` file.
/// - `codec`: (Optional) The compression algorithm to use. Supports `"brotli"` (default) or `"zstd"`.
/// - `quality`: (Optional) The compression quality level.
/// - `options`: (Optional) Additional codec-specific options.
///
/// # Examples
///
/// ```Rust
/// // Default: Minify + Brotli
/// let data = include_minified_wgsl_compressed!("shader.wgsl");
///
/// // Minify + Zstd with specific quality and codec
/// let data = include_minified_wgsl_compressed!(
///     "shader.wgsl",
///     codec = "zstd",
///     quality = 4
/// );
/// ```
#[cfg(feature = "wgsl_minify")]
#[proc_macro]
pub fn include_minified_wgsl_compressed(input: TokenStream) -> TokenStream {
    wgsl::include_minified_wgsl_compressed_impl(input)
}
