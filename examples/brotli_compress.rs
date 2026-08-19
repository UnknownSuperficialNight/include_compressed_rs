use std::io::Read;

use brotli::Decompressor;
use include_compressed::brotli_compress;

fn load_shader() -> (&'static str, &'static [u8]) {
    #[cfg(feature = "large_shader")]
    {
        let original = include_str!("large_shader.wgsl");

        let compressed_bytes = brotli_compress!("examples/large_shader.wgsl");

        (original, compressed_bytes)
    }

    #[cfg(not(feature = "large_shader"))]
    {
        let original = include_str!("../examples/simple_shader.wgsl");

        let compressed_bytes = brotli_compress!("examples/simple_shader.wgsl");

        (original, compressed_bytes)
    }
}

fn main() {
    let (original, compressed_bytes) = load_shader();

    println!("Original WGSL:");
    println!("{original}");
    println!();

    let mut decompressed = Vec::new();
    let mut decompressor = Decompressor::new(compressed_bytes, 4096);
    decompressor
        .read_to_end(&mut decompressed)
        .expect("shader decompression failed");

    let decompressed_str = String::from_utf8_lossy(&decompressed);

    println!("Decompressed WGSL:");
    println!("{decompressed_str}");
    println!();

    println!("Compressed bytes:");
    println!("{compressed_bytes:?}");
    println!();

    println!("Original size: {} bytes", original.len());
    println!("Compressed size: {} bytes", compressed_bytes.len());
    println!(
        "Reduction: {:.2}%",
        100.0 - (compressed_bytes.len() as f64 / original.len() as f64 * 100.0)
    );
}
