use syn::{
    ExprArray, Ident, LitBool, Token, braced,
    parse::{Parse, ParseStream},
};

/// Default value for `minify_whitespace`: removes unnecessary whitespace from WGSL source.
/// Set to `true` by default to reduce file size.
/// When `false`, original whitespace (indentation, line breaks) is preserved.
pub const DEFAULT_MINIFY_WHITESPACE: bool = true;

/// Default value for `minify_identifiers`: shortens WGSL identifier names.
/// Set to `true` by default to reduce file size.
/// When `false`, original identifier names are kept as-is.
pub const DEFAULT_MINIFY_IDENTIFIERS: bool = true;

/// Default value for `minify_syntax`: applies syntax optimizations.
/// Set to `true` by default to reduce file size.
/// When `false`, original syntax structure is preserved.
pub const DEFAULT_MINIFY_SYNTAX: bool = true;

/// Default value for `tree_shaking`: performs dead code elimination.
/// Set to `false` by default as this requires AST analysis and may impact shader behavior unpredictably.
/// When `true`, only referenced code is included in the output.
pub const DEFAULT_TREE_SHAKING: bool = false;

/// Default value for `mangle_external_bindings`: renames uniform/storage variable bindings directly.
/// Set to `true` by default to allow name mangling of external bindings.
/// When `false`, original binding names are preserved even if they conflict with local code.
pub const DEFAULT_MANGLE_EXTERNAL_BINDINGS: bool = true;

/// Default value for `preserve_uniform_struct_types`: preserves struct type names used in uniform declarations.
/// Set to `false` by default (allows renaming/optimizing).
/// When `true`, original struct type names are kept intact in uniform declarations.
pub const DEFAULT_PRESERVE_UNIFORM_STRUCT_TYPES: bool = false;

/// Default value for `source_map`: generates a source map (.map file) for the minified WGSL.
/// Set to `false` by default as source maps increase binary size without providing runtime benefits.
/// When `true`, a separate mapping file is generated that tracks original position mappings.
pub const DEFAULT_SOURCE_MAP: bool = false;

/// Default value for `source_map_sources`: includes source file paths in the source map.
/// Only relevant when `source_map` is enabled.
/// Set to `false` by default to minimize source map size.
/// When `true`, full source file paths are included, increasing source map size significantly.
pub const DEFAULT_SOURCE_MAP_SOURCES: bool = false;

/// Input structure for parsing minification options from the `include_minified_wgsl!` macro.
///
/// This struct represents the parse tree for the optional configuration block of the
/// `include_minified_wgsl!` macro. Each field corresponds to a minifier option that can be
/// customized in the macro invocation.
///
/// Fields marked with `Option<T>` will use their respective default values if not specified
/// by the user in the macro arguments.
///
/// # Example Macro Usage
///
/// ```rust,no_run
/// include_minified_wgsl!(
///     "shader.wgsl",
///     minify_whitespace = true,
///     keep_names = ["foo", "bar"],
/// );
/// ```
///
pub struct MinirayInput {
    /// Option to control whitespace removal (unnecessary whitespace deleted).
    /// Matches `--minify-whitespace` option.
    /// If `Some(LitBool::True)`, whitespace will be removed.
    /// If `Some(LitBool::False)`, original whitespace is preserved.
    /// If `None`, uses [DEFAULT_MINIFY_WHITESPACE].
    pub minify_whitespace: Option<LitBool>,

    /// Option to control identifier name shortening.
    /// Matches `--minify-identifiers` option.
    /// If `Some(LitBool::True)`, identifiers are shortened in the output.
    /// If `Some(LitBool::False)`, original identifier names are kept as-is.
    /// If `None`, uses [DEFAULT_MINIFY_IDENTIFIERS].
    pub minify_identifiers: Option<LitBool>,

    /// Option to control syntax optimizations.
    /// Matches `--minify-syntax` option.
    /// If `Some(LitBool::True)`, syntax is optimized in the output.
    /// If `Some(LitBool::False)`, original syntax structure is preserved.
    /// If `None`, uses [DEFAULT_MINIFY_SYNTAX].
    pub minify_syntax: Option<LitBool>,

    /// Option to enable tree shaking (removal of unused code).
    /// If `Some(LitBool::True)`, unused functions/variables are stripped.
    /// If `Some(LitBool::False)`, all code is included regardless of usage.
    /// If `None`, uses [DEFAULT_TREE_SHAKING].
    pub tree_shaking: Option<LitBool>,

    /// Option to control uniform/storage variable name mangling directly.
    /// If `Some(LitBool::True)`, bindings may be renamed during minification.
    /// If `Some(LitBool::False)`, original binding names are preserved.
    /// If `None`, uses [DEFAULT_MANGLE_EXTERNAL_BINDINGS].
    pub mangle_external_bindings: Option<LitBool>,

    /// Option to preserve struct type names used in uniform declarations.
    /// If `Some(LitBool::True)`, struct type names are kept intact.
    /// If `Some(LitBool::False)`, types may be renamed/obfuscated.
    /// If `None`, uses [DEFAULT_PRESERVE_UNIFORM_STRUCT_TYPES].
    pub preserve_uniform_struct_types: Option<LitBool>,

    /// Comma-separated string literal values specifying names to preserve (keep unminified).
    /// Matches `--keep-names <names>` option. Each element must be a string literal.
    /// Non-literal expressions are ignored.
    /// Empty array is equivalent to no names being preserved.
    pub keep_names: Option<ExprArray>,

    /// Option to enable source map generation (.map file output).
    /// If `Some(LitBool::True)`, a source map will be generated.
    /// If `Some(LitBool::False)`, no source map is created.
    /// If `None`, uses [DEFAULT_SOURCE_MAP].
    pub source_map: Option<LitBool>,

    /// Option to include original source content in the source map.
    /// Matches `--source-map-sources` option. Only relevant when `source_map` is enabled.
    /// If `Some(LitBool::True)`, full source file paths are included (increases size).
    /// If `Some(LitBool::False)`, minimal source information is included.
    /// If `None`, uses [DEFAULT_SOURCE_MAP_SOURCES].
    pub source_map_sources: Option<LitBool>,
}

/// Parse implementation for [`MinirayInput`].
///
/// This implementation handles parsing the struct syntax within the macro invocation,
/// extracting field names and their boolean/literal values. It validates that all fields
/// correspond to known minification options and returns an error for unknown fields.
///
/// # Syntax
///
/// ```text
/// MinirayInput {
///     field_name = value (, ...)?
/// }
/// ```
impl Parse for MinirayInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse and validate the struct identifier name
        let struct_ident: Ident = input.parse()?;
        if struct_ident != "MinirayInput" {
            return Err(syn::Error::new(
                struct_ident.span(),
                "Expected `MinirayInput`",
            ));
        }

        // Enter the brace-delimited body of the struct
        let content;
        braced!(content in input);

        // Initialize all optional fields to None (will use defaults if still None)
        let mut minify_whitespace = None;
        let mut minify_identifiers = None;
        let mut minify_syntax = None;
        let mut tree_shaking = None;
        let mut mangle_external_bindings = None;
        let mut preserve_uniform_struct_types = None;
        let mut keep_names = None;
        let mut source_map = None;
        let mut source_map_sources = None;

        // Parse all fields until the closing brace
        while !content.is_empty() {
            // Parse field name as identifier
            let field: Ident = content.parse()?;

            // Parse the colon separator
            let _: Token![:] = content.parse()?;

            // Match on field name and set corresponding option
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
                    ));
                }
            }

            // Handle optional comma between fields
            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            }
        }

        // Return the constructed struct with parsed (or default) values
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

/// Builds a JSON string of minification options by merging global defaults with user overrides.
///
/// This function constructs the configuration object passed to the miniray C API.
/// - When `input` is `None`, all fields use their [DEFAULT_*] values.
/// - When `input` is `Some`, each field uses the user-specified value if present,
///   otherwise falls back to the corresponding default.
///
/// The resulting JSON string matches the format expected by the miniray library.
///
/// # Example Output
///
/// Given a partial user input:
/// ```rust,no_run
/// minify_whitespace = false,
/// mangle_external_bindings = false,
/// ```
///
/// The output JSON would be (minified for brevity):
/// ```json
/// {"minifyWhitespace":false,"minifyIdentifiers":true,"minifySyntax":true,"treeShaking":false,"mangleExternalBindings":false,"preserveUniformStructTypes":false,"sourceMap":false,"sourceMapSources":false}
/// ```
pub fn build_options_json(input: Option<&MinirayInput>) -> String {
    use serde_json::json;

    // Create a mutable JSON object to populate with options
    let mut obj = serde_json::Map::new();

    // Helper closure that retrieves user-specified bool if provided, otherwise uses default.
    let get_bool = |user_opt: Option<&LitBool>, default: bool| -> bool {
        user_opt.map_or(default, |b| b.value)
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

    if let Some(arr) = input.and_then(|i| i.keep_names.as_ref()) {
        // Extract string literal values from the array elements
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
