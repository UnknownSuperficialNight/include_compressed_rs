use std::io::Read;

use brotli::Decompressor;

#[cfg(feature = "wgsl_minify")]
use include_compressed::include_minified_wgsl_brotli;

#[cfg(feature = "wgsl_minify")]
fn main() {
    let original = include_str!("simple_shader.wgsl");

    // Minify and compress the WGSL shader
    let compressed_bytes = include_minified_wgsl_brotli!("examples/simple_shader.wgsl");

    let mut decompressed = Vec::new();
    let mut decompressor = Decompressor::new(compressed_bytes.as_slice(), 4096);
    decompressor
        .read_to_end(&mut decompressed)
        .expect("shader decompression failed");

    let decompressed_str = String::from_utf8_lossy(&decompressed);

    println!("Original WGSL:");
    println!("{original}");
    println!();

    println!("Decompressed WGSL:");
    println!("{decompressed_str}");
    println!();

    println!("Original size: {} bytes", original.len());
    println!("Compressed size: {} bytes", compressed_bytes.len());
    println!(
        "Size reduction: {:.2}%",
        100.0 - (compressed_bytes.len() as f64 / original.len() as f64 * 100.0)
    );
}

#[cfg(not(feature = "wgsl_minify"))]
fn main() {
    eprintln!("========================================");
    eprintln!("  This example requires wgsl_minify     ");
    eprintln!("========================================");
    eprintln!();
    eprintln!("Run:");
    eprintln!("  cargo run --example include_minified_wgsl_brotli --features wgsl_minify");
}
