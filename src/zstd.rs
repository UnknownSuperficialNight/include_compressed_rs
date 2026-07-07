use proc_macro::TokenStream;
use quote::quote;
use std::io::Write;

use crate::parsing::{read_file, resolve_path, PathOptionInput};

/// Default Zstd compression level to MAX (11)
const DEFAULT_ZSTD_QUALITY: i32 = 11;

/// Compresses the given file path to zstd and returns the compressed bytes as a literal
///
/// Macro usage: `zstd_compress!("path", quality)`
///
/// - `path` - The path to the file to compress.
/// - `quality` - The compression quality (optional, defaults to [`DEFAULT_ZSTD_QUALITY`]).
pub fn zstd_compress_impl(input: TokenStream) -> TokenStream {
    // Parse using the generic helper.
    let PathOptionInput { path, option } = syn::parse_macro_input!(input as PathOptionInput);

    // Resolve and read the file.
    let abs_path = resolve_path(&path);
    let data = read_file(&abs_path);

    // The optional integer is the quality.
    let quality = option
        .map(|i| i.base10_parse::<i32>().unwrap())
        .unwrap_or(DEFAULT_ZSTD_QUALITY);

    // Perform Zstd compression
    let mut compressed = Vec::new();
    {
        // `zstd::stream::Encoder` writes compressed data into `compressed`.
        let mut encoder = zstd::stream::Encoder::new(&mut compressed, quality)
            .expect("failed to create Zstd encoder");
        encoder.write_all(&data).expect("Zstd compression failed");
        encoder.finish().expect("failed to finalize Zstd stream");
    }

    // Emit the compressed data as a byte‑slice literal.
    quote! { &[#(#compressed),*] }.into()
}
