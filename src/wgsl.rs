// All code in this module is only compiled if the `wgsl_minify` feature is enabled.
#![cfg(feature = "wgsl_minify")]

use proc_macro::TokenStream;
use quote::quote;
use std::{
    env,
    ffi::{CStr, CString},
    fs,
    io::Write,
    os::raw::{c_char, c_int, c_void},
    path::PathBuf,
    ptr,
};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    ExprArray, Ident, LitBool, LitInt, LitStr, Token,
};

// DEFAULTS

const DEFAULT_BROTLI_QUALITY: u32 = 11;

/// Default minification settings used if the user doesn't provide a field.
/// Values match the Options Reference table exactly.
const DEFAULT_MINIFY_WHITESPACE: bool = true;
const DEFAULT_MINIFY_IDENTIFIERS: bool = true;
const DEFAULT_MINIFY_SYNTAX: bool = true;
const DEFAULT_TREE_SHAKING: bool = false;
const DEFAULT_MANGLE_EXTERNAL_BINDINGS: bool = true;
const DEFAULT_PRESERVE_UNIFORM_STRUCT_TYPES: bool = false;
const DEFAULT_SOURCE_MAP: bool = false;
const DEFAULT_SOURCE_MAP_SOURCES: bool = false;

// PARSING

pub struct MinirayInput {
    pub minify_whitespace: Option<LitBool>,
    pub minify_identifiers: Option<LitBool>,
    pub minify_syntax: Option<LitBool>,
    pub tree_shaking: Option<LitBool>,
    pub mangle_external_bindings: Option<LitBool>,
    pub preserve_uniform_struct_types: Option<LitBool>,
    pub keep_names: Option<ExprArray>,
    pub source_map: Option<LitBool>,
    pub source_map_sources: Option<LitBool>,
}

impl Parse for MinirayInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_ident: Ident = input.parse()?;
        if struct_ident != "MinirayInput" {
            return Err(syn::Error::new(
                struct_ident.span(),
                "Expected `MinirayInput`",
            ));
        }
        let content;
        braced!(content in input);

        let mut minify_whitespace = None;
        let mut minify_identifiers = None;
        let mut minify_syntax = None;
        let mut tree_shaking = None;
        let mut mangle_external_bindings = None;
        let mut preserve_uniform_struct_types = None;
        let mut keep_names = None;
        let mut source_map = None;
        let mut source_map_sources = None;

        while !content.is_empty() {
            let field: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            match field.to_string().as_str() {
                "minify_whitespace" => minify_whitespace = Some(content.parse()?),
                "minify_identifiers" => minify_identifiers = Some(content.parse()?),
                "minify_syntax" => minify_syntax = Some(content.parse()?),
                "tree_shaking" => tree_shaking = Some(content.parse()?),
                "mangle_external_bindings" => mangle_external_bindings = Some(content.parse()?),
                "preserve_uniform_struct_types" => {
                    preserve_uniform_struct_types = Some(content.parse()?)
                }
                "keep_names" => keep_names = Some(content.parse()?),
                "source_map" => source_map = Some(content.parse()?),
                "source_map_sources" => source_map_sources = Some(content.parse()?),
                _ => {
                    return Err(syn::Error::new(
                        field.span(),
                        "Unknown field in MinirayInput",
                    ))
                }
            }
            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            }
        }

        Ok(MinirayInput {
            minify_whitespace,
            minify_identifiers,
            minify_syntax,
            tree_shaking,
            mangle_external_bindings,
            preserve_uniform_struct_types,
            keep_names,
            source_map,
            source_map_sources,
        })
    }
}

pub struct WgslArgs {
    pub path: LitStr,
    pub options: Option<MinirayInput>,
}

impl Parse for WgslArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        let options = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(WgslArgs { path, options })
    }
}

pub struct WgslBrotliArgs {
    pub path: LitStr,
    pub quality: Option<LitInt>,
    pub options: Option<MinirayInput>,
}

impl Parse for WgslBrotliArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        let mut quality = None;
        let mut options = None;

        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.peek(LitInt) {
                quality = Some(input.parse()?);
                if input.peek(Token![,]) {
                    let _: Token![,] = input.parse()?;
                    options = Some(input.parse()?);
                }
            } else if input.peek(Ident) {
                options = Some(input.parse()?);
            }
        }
        Ok(WgslBrotliArgs {
            path,
            quality,
            options,
        })
    }
}

// LOGIC

/// Builds a JSON string of minification options, merging global defaults with any
/// user-provided overrides. When `input` is `None`, all fields take their defaults.
/// When `input` is `Some`, each field uses the user value if present, else the default.
fn build_options_json(input: Option<&MinirayInput>) -> String {
    use serde_json::json;
    let mut obj = serde_json::Map::new();

    // Returns the user-supplied bool if provided, otherwise falls back to `default`.
    // Works uniformly whether `input` is Some or None via Option::and_then.
    let get_bool = |user_opt: Option<&LitBool>, default: bool| -> bool {
        user_opt.map(|b| b.value).unwrap_or(default)
    };

    obj.insert(
        "minifyWhitespace".into(),
        json!(get_bool(
            input.and_then(|i| i.minify_whitespace.as_ref()),
            DEFAULT_MINIFY_WHITESPACE
        )),
    );
    obj.insert(
        "minifyIdentifiers".into(),
        json!(get_bool(
            input.and_then(|i| i.minify_identifiers.as_ref()),
            DEFAULT_MINIFY_IDENTIFIERS
        )),
    );
    obj.insert(
        "minifySyntax".into(),
        json!(get_bool(
            input.and_then(|i| i.minify_syntax.as_ref()),
            DEFAULT_MINIFY_SYNTAX
        )),
    );
    obj.insert(
        "treeShaking".into(),
        json!(get_bool(
            input.and_then(|i| i.tree_shaking.as_ref()),
            DEFAULT_TREE_SHAKING
        )),
    );
    obj.insert(
        "mangleExternalBindings".into(),
        json!(get_bool(
            input.and_then(|i| i.mangle_external_bindings.as_ref()),
            DEFAULT_MANGLE_EXTERNAL_BINDINGS
        )),
    );
    obj.insert(
        "preserveUniformStructTypes".into(),
        json!(get_bool(
            input.and_then(|i| i.preserve_uniform_struct_types.as_ref()),
            DEFAULT_PRESERVE_UNIFORM_STRUCT_TYPES
        )),
    );
    obj.insert(
        "sourceMap".into(),
        json!(get_bool(
            input.and_then(|i| i.source_map.as_ref()),
            DEFAULT_SOURCE_MAP
        )),
    );
    obj.insert(
        "sourceMapSources".into(),
        json!(get_bool(
            input.and_then(|i| i.source_map_sources.as_ref()),
            DEFAULT_SOURCE_MAP_SOURCES
        )),
    );

    // `keepNames` is only emitted when the user explicitly provides the array.
    if let Some(arr) = input.and_then(|i| i.keep_names.as_ref()) {
        let names: Vec<String> = arr
            .elems
            .iter()
            .filter_map(|e| {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = e
                {
                    Some(s.value())
                } else {
                    None
                }
            })
            .collect();
        obj.insert("keepNames".into(), json!(names));
    }

    serde_json::to_string(&obj).unwrap()
}

unsafe extern "C" {
    fn miniray_minify(
        source: *const c_char,
        source_len: c_int,
        options: *const c_char,
        options_len: c_int,
        out_code: *mut *mut c_char,
        out_code_len: *mut c_int,
        out_json: *mut *mut c_char,
        out_json_len: *mut c_int,
    ) -> c_int;
    fn miniray_free(ptr: *mut c_void);
}

fn run_miniray(data: &[u8], options_json: &str) -> Vec<u8> {
    let opts_cstr = CString::new(options_json).unwrap();
    let mut out_code: *mut c_char = ptr::null_mut();
    let mut out_code_len: c_int = 0;
    let mut out_json: *mut c_char = ptr::null_mut();
    let mut out_json_len: c_int = 0;

    let rc = unsafe {
        miniray_minify(
            data.as_ptr() as *const c_char,
            data.len() as c_int,
            opts_cstr.as_ptr(),
            opts_cstr.as_bytes().len() as c_int,
            &mut out_code,
            &mut out_code_len,
            &mut out_json,
            &mut out_json_len,
        )
    };

    if rc != 0 || out_code.is_null() {
        let msg = if !out_json.is_null() {
            let s = unsafe { CStr::from_ptr(out_json).to_string_lossy().into_owned() };
            unsafe { miniray_free(out_json as *mut c_void) };
            s
        } else {
            format!("miniray_minify failed: {}", rc)
        };
        panic!("{}", msg);
    }

    let result =
        unsafe { std::slice::from_raw_parts(out_code as *const u8, out_code_len as usize) }
            .to_vec();
    unsafe {
        miniray_free(out_code as *mut c_void);
        if !out_json.is_null() {
            miniray_free(out_json as *mut c_void);
        }
    }
    result
}

pub fn include_minified_wgsl_impl(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as WgslArgs);
    let abs_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(args.path.value());
    let data = fs::read(&abs_path).unwrap();
    let minified = run_miniray(&data, &build_options_json(args.options.as_ref()));
    let s = String::from_utf8(minified).unwrap();
    quote!(#s).into()
}

pub fn include_minified_wgsl_brotli_impl(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as WgslBrotliArgs);
    let abs_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(args.path.value());
    let data = fs::read(&abs_path).unwrap();
    let minified = run_miniray(&data, &build_options_json(args.options.as_ref()));

    // Use as_ref() so `args.quality` (Option<LitInt>) is not moved; parse quality or use default.
    let quality = args
        .quality
        .as_ref()
        .map(|q| q.base10_parse::<u32>().unwrap())
        .unwrap_or(DEFAULT_BROTLI_QUALITY);

    let mut compressed = Vec::new();
    {
        let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, quality, 22);
        writer.write_all(&minified).unwrap();
    }

    quote!(&[#(#compressed),*]).into()
}
