# include_compressed

A Rust proc-macro library for embedding compressed, minified WGSL shaders and compressed files at compile time.

## Overview

This library provides procedural macros for compressing and embedding files at compile time:

- **brotli_compress!** - Compress any file using Brotli (no minification)
- **zstd_compress!** - Compress any file using Zstd (no minification)
- **include_minified_wgsl!** - Minify WGSL shaders with Brotli compression (wgsl_minify feature required)
- **include_minified_wgsl_compressed!** - Minify WGSL shaders with Brotli or Zstd compression (codec selectable; wgsl_minify feature required)

All data is compressed then embedded directly in the compiled binary.

Decompression is not provided by this crate since this is a proc-macro crate. You must handle decompression using the `brotli`, `zstd` or decompression crates separately.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
include_compressed = "0.1.0"
```

or with wgsl_minification:

```
[dependencies]
include_compressed = { version = "0.1.0", features = ["wgsl_minify"] }
```

## Usage

### Brotli Compression

```rust
use include_compressed::brotli_compress;

// Compress a WGSL shader file with Brotli compression (no minification)
let compressed_bytes = brotli_compress!("examples/simple_shader.wgsl");

println!("Compressed bytes: {}", compressed_bytes.len());
println!("Compressed shader: {:?}", compressed_bytes);
```

> **Note:** Brotli decompression is not provided by this crate. Use the `brotli` crate separately:
>
> ```rust
> use brotli::Decompressor;
>
> let mut decompressed = Vec::new();
> let mut decompressor = Decompressor::new(
>     compressed_bytes.as_slice(),
>     4096 // window size
> );
> decompressor.read_to_end(&mut decompressed).expect("decompression failed");
> ```

### Zstd Compression

```rust
use include_compressed::zstd_compress;

// Compress a WGSL shader file with Zstd compression (no minification)
let compressed_bytes = zstd_compress!("examples/simple_shader.wgsl");

println!("Compressed bytes: {}", compressed_bytes.len());
```

> **Note:** Zstd decompression is not provided by this crate. Use the `zstd` crate separately:
>
> ```rust
> use zstd::Decoder;
>
> let mut decompressed = Vec::new();
> let mut decoder = Decoder::new(compressed_bytes.as_slice())
>     .expect("failed to create Zstd decoder");
> decoder.read_to_end(&mut decompressed).expect("decompression failed");
> ```

### WGSL Minification

```rust
#[cfg(feature = "wgsl_minify")]
use include_compressed::include_minified_wgsl;

#[cfg(feature = "wgsl_minify")]
// Minify a WGSL shader file (uses Brotli compression internally)
let minified = include_minified_wgsl!("examples/simple_shader.wgsl");

println!("Minified shader: {} bytes", minified.len());
```

> **Note:** Use the `wgsl_minify` feature to enable WGSL minification macros. The `serde_json` dependency is automatically pulled in when this feature is enabled.

### WGSL Minification and Compression

```rust
#[cfg(feature = "wgsl_minify")]
use include_compressed::include_minified_wgsl_compressed;

#[cfg(feature = "wgsl_minify")]
// Minify and compress the WGSL shader using Brotli compression (default)
let compressed_brotli = include_minified_wgsl_compressed!("examples/simple_shader.wgsl");

// You can also specify compression codec and quality:
let compressed_zstd = include_minified_wgsl_compressed!(
    "examples/simple_shader.wgsl",
    codec = "zstd",
    quality = 4
);

println!("Compressed shader: {} bytes", compressed_brotli.len());
```

This macro combines WGSL shader minification with compression in one step. Supports:
- **Brotli**: Brotli compression (max quality 11)
- **Zstd**: Use `codec = "zstd"` to compress with Zstd (max quality 22)
- **Quality**: Use `quality = N` to set compression level (1-22 for both codecs)

## API Reference

| Macro | Description |
|-------|-------------|
| `brotli_compress!` | Embed a file with Brotli compression (no minification) |
| `zstd_compress!` | Embed a file with Zstd compression (no minification) |
| `include_minified_wgsl!` | Minify a WGSL shader with Brotli compression |
| `include_minified_wgsl_compressed!` | Minify WGSL and compress (codec: Brotli or Zstd) |

## Examples

Run examples with:

```bash
# Brotli compression
cargo run --example brotli_compress

# Zstd compression
cargo run --example zstd_compress

# WGSL minification
cargo run --example include_minified_wgsl --features wgsl_minify

# Minify and compress with Brotli
cargo run --example include_minified_wgsl_compressed --features wgsl_minify

# Minify and compress with Zstd
cargo run --example include_minified_wgsl_compressed_zstd --features wgsl_minify
```

## License

Licensed under Apache License, Version 2.0.
