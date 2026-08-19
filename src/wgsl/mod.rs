mod external_unsafe;
mod miniray_parsing;
mod unsafe_run_miniray;
mod wgsl_parsing;

use proc_macro::TokenStream;
use quote::quote;
use std::{env, fs, io::Write, path::PathBuf};

use crate::{
    parsing::CodecChoice,
    wgsl::{
        miniray_parsing::build_options_json,
        unsafe_run_miniray::run_miniray,
        wgsl_parsing::{WgslArgs, WgslCompressedArgs},
    },
};

/// Default Brotli compression quality level.
/// Range: 0-11, where higher values produce better compression at the cost of more CPU time.
/// This is a standard value used by the brotli crate.
const DEFAULT_BROTLI_QUALITY: u32 = 11;

/// Compression quality level for zstd codec. Controls compression ratio vs speed.
/// Range: 0-22, higher values produce better compression at the cost of more CPU time and memory.
/// Level 22 is a high-quality default that balances compression ratio with encoding speed.
const DEFAULT_ZSTD_QUALITY: i32 = 22;

/// Minifies a WGSL shader source file using miniray.
///
/// This macro processes a WGSL shader file by:
/// 1. Reading the WGSL source file from disk
/// 2. Passing it to miniray for minification
/// 3. Returning the minified WGSL as a string literal
///
/// This is the internal implementation used by `include_minified_wgsl!` macro.
///
/// # Arguments
/// * `input` - TokenStream containing the WGSL file path and optional configuration
///
/// # Returns
/// A token stream containing the minified WGSL shader source code as a string literal.
///
/// # Example
/// ```ignore
/// let code = include_minified_wgsl!("examples/shader.wgsl");
/// ```
pub fn include_minified_wgsl_impl(input: TokenStream) -> TokenStream {
    // Parse macro arguments into WgslArgs structure.
    // Panics on invalid syntax, which indicates malformed macro invocation.
    let args = syn::parse_macro_input!(input as WgslArgs);

    // Construct absolute path to WGSL file using cargo manifest directory.
    // Panics if CARGO_MANIFEST_DIR is not set, which indicates a build environment error.
    let abs_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(args.path.value());

    // Read WGSL source file from disk.
    // Panics if file not found or unreadable - this is acceptable as
    // a missing source file is a fatal build-time error that should fail fast.
    let data = fs::read(&abs_path)
        .unwrap_or_else(|_| panic!("Failed to read WGSL source file: {}", abs_path.display()));

    // Build minification options JSON string with user overrides applied to defaults.
    let minified = run_miniray(&data, &build_options_json(args.options.as_ref()));

    // Convert minified bytes to UTF-8 string. Panics if conversion fails,
    // which indicates corrupted or malformed WGSL output from miniray.
    let s = String::from_utf8(minified).unwrap_or_else(|_| {
        panic!("Minified WGSL output is not valid UTF-8: corrupted shader data")
    });

    // Use quote! to expand the string directly into the macro invocation site.
    // The resulting string literal becomes part of the Rust source code.
    quote!(#s).into()
}

/// Compresses and minifies a WGSL shader source file using miniray with optional codec.
///
/// This macro processes a WGSL shader file by:
/// 1. Reading the WGSL source file from disk
/// 2. Passing it to miniray for minification
/// 3. Compressing the minified output using the specified codec (Brotli or Zstd)
/// 4. Returning the compressed bytes as a byte array reference
///
/// This is the internal implementation used by `include_minified_wgsl_compressed!` macro.
///
/// # Arguments
/// * `input` - TokenStream containing:
///   - WGSL file path
///   - Optional codec (Brotli or Zstd)
///   - Optional compression quality
///   - Optional miniray configuration options
///
/// # Returns
/// A token stream containing `&[u8]` reference to compressed shader bytes.
/// The returned array contains raw bytes of the compressed WGSL shader.
///
/// # Example
/// ```ignore
/// // Compresses `examples/shader.wgsl` using Brotli
/// let bytes = include_minified_wgsl_compressed!("examples/shader.wgsl");
///
/// // Compresses `examples/shader.wgsl` using Zstd with quality 5
/// let bytes = include_minified_wgsl_compressed!("examples/shader.wgsl", codec = "zstd", quality = 5);
/// ```
pub fn include_minified_wgsl_compressed_impl(input: TokenStream) -> TokenStream {
    // Parse macro arguments into WgslCompressedArgs structure.
    // Panics on invalid syntax, which indicates malformed macro invocation.
    let args = syn::parse_macro_input!(input as WgslCompressedArgs);

    // Construct absolute path to WGSL file using cargo manifest directory.
    // Panics if CARGO_MANIFEST_DIR is not set, which indicates a build environment error.
    let abs_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(args.path.value());

    // Read WGSL source file from disk.
    // Panics if file not found or unreadable - this is acceptable as
    // a missing source file is a fatal build-time error that should fail fast.
    let data = fs::read(&abs_path)
        .unwrap_or_else(|_| panic!("Failed to read WGSL source file: {}", abs_path.display()));

    // Step 1: Minify the WGSL source using configured options (same as non-compressed version).
    let minified = run_miniray(&data, &build_options_json(args.options.as_ref()));

    // Determine compression codec to use; default to Brotli if not specified.
    let codec = args.codec.unwrap_or(CodecChoice::Brotli);

    // Select appropriate compression and encode the minified WGSL.
    let compressed = match codec {
        // Brotli compression path
        CodecChoice::Brotli => {
            // Parse quality value to u32. Panics on invalid number format,
            // which indicates a malformed macro invocation.
            let quality = args
                .quality
                .map_or(DEFAULT_BROTLI_QUALITY, |q| q.base10_parse::<u32>().unwrap());

            // Create buffer for compressed output.
            // The Vec::new() buffer grows automatically; the 4096-byte initial window
            // provides a reasonable trade-off between memory usage and compression ratio.
            let mut compressed = Vec::new();

            // Use block scope to ensure writer is dropped after flush completes.
            {
                let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, quality, 22);

                writer.write_all(&minified).unwrap_or_else(|_| {
                    panic!("Failed to write minified WGSL to Brotli compressor")
                });
            }

            // Return the byte vector containing Brotli-compressed WGSL data.
            compressed
        }

        // Zstd compression path
        CodecChoice::Zstd => {
            // Parse quality value to i32. Panics on invalid number format,
            // which indicates a malformed macro invocation.
            let quality = args
                .quality
                .map_or(DEFAULT_ZSTD_QUALITY, |q| q.base10_parse::<i32>().unwrap());

            // Create buffer for compressed output.
            // The Vec::new() buffer grows automatically; the encoder manages its own internal window.
            let mut compressed = Vec::new();

            // Use block scope to ensure encoder is properly finalized and flushed.
            {
                let mut encoder =
                    zstd::Encoder::new(&mut compressed, quality).unwrap_or_else(|_| {
                        panic!("Failed to create Zstd encoder with quality {quality}")
                    });

                encoder
                    .write_all(&minified)
                    .unwrap_or_else(|_| panic!("Failed to write minified WGSL to Zstd encoder"));

                encoder.finish().unwrap_or_else(|_| {
                    panic!("Failed to finalize Zstd encoder - internal encoding error occurred")
                });
            }

            // Return the byte vector containing Zstd-compressed WGSL data.
            compressed
        }
    };

    // Use quote! to expand a byte array reference into the macro invocation site.
    // The result is `&[b1, b2, b3, ...]` - a static slice of bytes.
    quote!(&[#(#compressed),*]).into()
}
