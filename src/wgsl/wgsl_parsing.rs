use syn::{
    Ident, LitInt, LitStr, Token,
    parse::{Parse, ParseStream},
};

use crate::{parsing::CodecChoice, wgsl::miniray_parsing::MinirayInput};

/// Arguments structure for the [`include_minified_wgsl!`](crate::macro_include_minified_wgsl) macro.
///
/// This struct holds the parsed arguments for the basic (non-compressed) WGSL minification macro:
/// - `path`: Path to the WGSL source file to include
/// - `options`: Optional configuration via [`MinirayInput`]
///
/// The macro reads the WGSL file from disk, applies minification according to options,
/// and outputs the minified shader code directly into the Rust compilation unit.
pub struct WgslArgs {
    /// Path to the WGSL source file (relative to `CARGO_MANIFEST_DIR`).
    pub path: LitStr,

    /// Optional minification settings override.
    /// When provided, these values override the [DEFAULT_*] constants.
    /// When absent or None, all defaults from the DEFAULT section are applied.
    pub options: Option<MinirayInput>,
}

/// Parse implementation for [`WgslArgs`].
///
/// Parses the macro arguments in the format:
/// ```text
/// "path/to/file.wgsl" [, options]?
/// ```
///
/// The path argument is required and must be a string literal. The optional
/// `options` block (if present) contains a [`MinirayInput`] configuration.
impl Parse for WgslArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the required WGSL file path as a string literal
        let path: LitStr = input.parse()?;

        // Check if comma precedes optional options block
        let options = if input.peek(Token![,]) {
            // Consume comma and parse MinirayInput configuration
            let _: Token![,] = input.parse()?;
            Some(input.parse()?)
        } else {
            // No options specified, use defaults
            None
        };

        Ok(WgslArgs { path, options })
    }
}

/// Arguments structure for the [`include_minified_wgsl_compressed!`](crate::macro_include_minified_wgsl_compressed) macro.
///
/// This struct holds the parsed arguments for the compressed WGSL minification macro:
/// - `path`: Path to the WGSL source file to include
/// - `quality`: Compression quality level (codec-specific range)
/// - `codec`: Compression algorithm to use (Brotli or Zstd)
/// - `options`: Optional minification settings via [`MinirayInput`]
///
/// The macro reads the WGSL file, minifies it according to options, then compresses
/// the result using the specified codec before embedding into the Rust source.
pub struct WgslCompressedArgs {
    /// Path to the WGSL source file (relative to `CARGO_MANIFEST_DIR`).
    pub path: LitStr,

    /// Compression quality level for the chosen codec.
    /// - For Brotli: 0-11 (higher = better compression, more CPU time)
    /// - For Zstd: 0-22 (higher = better compression, more CPU/memory)
    ///
    /// If not specified, uses [DEFAULT_BROTLI_QUALITY] or [DEFAULT_ZSTD_QUALITY] respectively.
    pub quality: Option<LitInt>,

    /// Compression codec to use for the output.
    /// - `brotli`: Uses Brotli compression (good balance of speed/size)
    /// - `zstd`: Uses Zstandard compression (fast decompression, good size)
    ///
    /// If not specified, defaults to `brotli`.
    pub codec: Option<CodecChoice>,

    /// Optional minification settings override.
    /// Same semantics as [`WgslArgs::options`].
    pub options: Option<MinirayInput>,
}

/// Parse implementation for [`WgslCompressedArgs`].
///
/// Parses the macro arguments in the format:
/// ```text
/// "path/to/file.wgsl" [, codec = "codec", quality = N, options]?
/// ```
///
/// Required argument:
/// - `codec`: Either `"brotli"` or `"zstd"`
///
/// Optional arguments:
/// - `quality`: Compression quality level (number literal)
/// - `options`: [`MinirayInput`] configuration block
impl Parse for WgslCompressedArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the required WGSL file path as a string literal
        let path: LitStr = input.parse()?;

        // Initialize all optional fields
        let mut quality: Option<LitInt> = None;
        let mut codec: Option<CodecChoice> = None;
        let mut options: Option<MinirayInput> = None;

        // Parse comma-separated key-value pairs
        while input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;

            // Stop if we've reached the end after the comma
            if input.is_empty() {
                break;
            }

            // Parse option key and equals sign
            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            // Match on option key and parse corresponding value
            match key.to_string().as_str() {
                "codec" => {
                    // Parse codec as string literal and convert to CodecChoice enum
                    let lit: LitStr = input.parse()?;
                    codec = Some(
                        CodecChoice::from_str(&lit.value())
                            .map_err(|e| syn::Error::new(lit.span(), e))?,
                    );
                }
                "quality" => {
                    // Parse quality as integer literal
                    quality = Some(input.parse()?);
                }
                "options" => {
                    // Parse options block as MinirayInput
                    options = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected codec, quality, or options",
                    ));
                }
            }
        }

        Ok(Self {
            path,
            quality,
            codec,
            options,
        })
    }
}
