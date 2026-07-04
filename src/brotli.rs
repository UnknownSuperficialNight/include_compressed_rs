use proc_macro::TokenStream;
use quote::quote;
use std::{env, fs, io::Write, path::PathBuf};
use syn::{
    parse::{Parse, ParseStream},
    LitInt, LitStr, Token,
};

const DEFAULT_BROTLI_QUALITY: u32 = 11;

/// Macro input: "path", quality
pub struct BrotliInput {
    pub path: LitStr,
    pub quality: Option<LitInt>,
}

impl Parse for BrotliInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Path is mandatory and must come first
        let path: LitStr = input.parse()?;
        let mut quality = None;

        // Check for optional quality: , 11
        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.peek(LitInt) {
                quality = Some(input.parse()?);
            }
        }

        Ok(BrotliInput { path, quality })
    }
}

/// Macro: brotli_compress_impl!("path", quality)
pub fn brotli_compress_impl(input: TokenStream) -> TokenStream {
    let BrotliInput { path, quality } = syn::parse_macro_input!(input as BrotliInput);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let abs_path = PathBuf::from(manifest_dir).join(path.value());
    let data = fs::read(&abs_path)
        .unwrap_or_else(|e| panic!("failed to read file {}: {}", abs_path.display(), e));

    let quality = quality
        .map(|q| q.base10_parse::<u32>().unwrap())
        .unwrap_or(DEFAULT_BROTLI_QUALITY);

    let mut compressed = Vec::new();
    {
        let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, quality, 22);
        writer.write_all(&data).expect("brotli compression failed");
    }

    let out = quote! {
        /* Embedded with Brotli compression, quality = #quality */
        &[#(#compressed),*]
    };
    out.into()
}
