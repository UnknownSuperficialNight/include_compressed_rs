use std::io::Read;

use include_compressed::zstd_compress;

fn main() {
    let original = include_str!("simple_shader.wgsl");

    // Compress a WGSL shader file with Zstd compression, no minification
    let compressed_bytes = zstd_compress!("examples/simple_shader.wgsl");

    println!("Original WGSL:");
    println!("{original}");
    println!();

    let mut decompressed = Vec::new();
    let mut decoder = zstd::stream::Decoder::new(compressed_bytes.as_slice())
        .expect("failed to create Zstd decoder");
    decoder
        .read_to_end(&mut decompressed)
        .expect("Zstd decompression failed");

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
