use proc_macro::TokenStream;
use quote::quote;
use std::{env, fs, io::Write, path::PathBuf};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, LitInt, LitStr, Token,
};

/// Macro input: { path: "...", quality: ... }
pub struct BrotliInput {
    pub path: LitStr,
    pub quality: Option<LitInt>,
}

impl Parse for BrotliInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        let mut quality = None;
        let pairs = Punctuated::<syn::MetaNameValue, Token![,]>::parse_terminated(input)?;
        for pair in pairs {
            let key = pair
                .path
                .get_ident()
                .ok_or_else(|| syn::Error::new_spanned(&pair.path, "Expected identifier"))?
                .to_string();
            match key.as_str() {
                "path" => {
                    if let syn::Expr::Lit(expr_lit) = &pair.value {
                        if let syn::Lit::Str(s) = &expr_lit.lit {
                            path = Some(s.clone());
                        }
                    }
                }
                "quality" => {
                    if let syn::Expr::Lit(expr_lit) = &pair.value {
                        if let syn::Lit::Int(i) = &expr_lit.lit {
                            quality = Some(i.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(BrotliInput {
            path: path
                .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "Missing `path`"))?,
            quality,
        })
    }
}

/// Macro: include_brotli_bytes!({ path: "...", quality: ... })
pub fn include_brotli_bytes_impl(input: TokenStream) -> TokenStream {
    let BrotliInput { path, quality } = syn::parse_macro_input!(input as BrotliInput);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let abs_path = PathBuf::from(manifest_dir).join(path.value());
    let data = fs::read(&abs_path)
        .unwrap_or_else(|e| panic!("failed to read file {}: {}", abs_path.display(), e));
    let quality = quality
        .map(|q| q.base10_parse::<u32>().unwrap())
        .unwrap_or(11);

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

/// Macro input: { expr: ..., quality: ... }
pub struct BrotliCompressInput {
    pub expr: Expr,
    pub quality: Option<LitInt>,
}

impl Parse for BrotliCompressInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut expr = None;
        let mut quality = None;
        let pairs = Punctuated::<syn::MetaNameValue, Token![,]>::parse_terminated(input)?;
        for pair in pairs {
            let key = pair
                .path
                .get_ident()
                .ok_or_else(|| syn::Error::new_spanned(&pair.path, "Expected identifier"))?
                .to_string();
            match key.as_str() {
                "expr" => {
                    expr = Some(pair.value.clone());
                }
                "quality" => {
                    if let syn::Expr::Lit(expr_lit) = &pair.value {
                        if let syn::Lit::Int(i) = &expr_lit.lit {
                            quality = Some(i.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(BrotliCompressInput {
            expr: expr
                .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "Missing `expr`"))?,
            quality,
        })
    }
}

/// Macro: brotli_compress!({ expr: ..., quality: ... })
pub fn brotli_compress_impl(input: TokenStream) -> TokenStream {
    let BrotliCompressInput { expr, quality } =
        syn::parse_macro_input!(input as BrotliCompressInput);
    let quality = quality
        .map(|q| q.base10_parse::<u32>().unwrap())
        .unwrap_or(5);

    quote! {{
        /* Brotli compression, quality = #quality */
        use brotli::CompressorWriter;
        use std::io::Write;
        let mut compressed = Vec::new();
        {
            let mut writer = CompressorWriter::new(&mut compressed, 4096, #quality, 22);
            writer.write_all(&(#expr)).expect("brotli compression failed");
        }
        compressed
    }}
    .into()
}
