use proc_macro::TokenStream;
use quote::quote;
use std::io::Write;

use crate::parsing::{PathOptionInput, read_file, resolve_path};

/// Default Brotli compression level to MAX (11)
const DEFAULT_BROTLI_QUALITY: u32 = 11;

/// Compresses the given file path to brotli and returns the compressed bytes as a literal
///
/// Macro usage: `brotli_compress!("path", quality)`
///
/// - `path` - The path to the file to compress.
/// - `quality` - The compression quality (optional, defaults to [`DEFAULT_BROTLI_QUALITY`]).
pub fn brotli_compress_impl(input: TokenStream) -> TokenStream {
    // Parse using the generic helper.
    let PathOptionInput { path, option } = syn::parse_macro_input!(input as PathOptionInput);

    // Resolve and read the file.
    let abs_path = resolve_path(&path);
    let data = read_file(&abs_path);

    // The optional integer is the quality.
    let quality = option.map_or(DEFAULT_BROTLI_QUALITY, |i| i.base10_parse::<u32>().unwrap());

    // Perform compression.
    let mut compressed = Vec::new();
    {
        let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, quality, 22);
        writer.write_all(&data).expect("brotli compression failed");
    }

    // Emit the compressed data as a byte‑slice literal.
    quote! { &[#(#compressed),*] }.into()
}
