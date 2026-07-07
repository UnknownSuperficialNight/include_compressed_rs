use proc_macro::TokenStream;
use quote::quote;
use std::{env, fs, io::Write, path::PathBuf};
use syn::{
    parse::{Parse, ParseStream},
    LitInt, LitStr, Token,
};

/// Generic macro input: mandatory string literal followed by an optional integer.
pub struct PathOptionInput {
    pub path: LitStr,
    pub option: Option<LitInt>,
}

impl Parse for PathOptionInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // First token must be a string literal – the file path.
        let path: LitStr = input.parse()?;
        let mut option = None;

        // Optional “, <int>”
        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.peek(LitInt) {
                option = Some(input.parse()?);
            }
        }

        Ok(PathOptionInput { path, option })
    }
}

/// Resolve a `PathOptionInput` into an absolute path inside the crate.
pub fn resolve_path(input_path: &LitStr) -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join(input_path.value())
}

/// Read the whole file and panic with a clear message on error.
pub fn read_file(path: &PathBuf) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("failed to read file {}: {}", path.display(), e))
}
