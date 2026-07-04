extern crate proc_macro;

mod brotli;

use proc_macro::TokenStream;

#[proc_macro]
pub fn include_brotli_bytes(input: TokenStream) -> TokenStream {
    brotli::include_brotli_bytes_impl(input)
}

#[proc_macro]
pub fn brotli_compress(input: TokenStream) -> TokenStream {
    brotli::brotli_compress_impl(input)
}

#[cfg(feature = "wgsl_minify")]
mod wgsl;
#[cfg(feature = "wgsl_minify")]
#[proc_macro]
pub fn include_minified_wgsl(input: TokenStream) -> TokenStream {
    wgsl::include_minified_wgsl_impl(input)
}
#[cfg(feature = "wgsl_minify")]
#[proc_macro]
pub fn include_minified_wgsl_brotli(input: TokenStream) -> TokenStream {
    wgsl::include_minified_wgsl_brotli_impl(input)
}
