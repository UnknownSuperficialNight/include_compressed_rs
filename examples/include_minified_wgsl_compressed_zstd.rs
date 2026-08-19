use std::io::Read;

#[cfg(feature = "wgsl_minify")]
use include_compressed::include_minified_wgsl_compressed;
#[cfg(feature = "wgsl_minify")]
use zstd::Decoder;

#[cfg(feature = "wgsl_minify")]
fn load_shader() -> (&'static str, &'static [u8]) {
    #[cfg(feature = "large_shader")]
    {
        let original = include_str!("large_shader.wgsl");

        let compressed_bytes = include_minified_wgsl_compressed!(
            "examples/large_shader.wgsl",
            codec = "zstd",
            quality = 4
        );

        (original, compressed_bytes)
    }

    #[cfg(not(feature = "large_shader"))]
    {
        let original = include_str!("../examples/simple_shader.wgsl");

        let compressed_bytes = include_minified_wgsl_compressed!(
            "examples/simple_shader.wgsl",
            codec = "zstd",
            quality = 4
        );

        (original, compressed_bytes)
    }
}

#[cfg(feature = "wgsl_minify")]
fn main() {
    let (original, compressed_bytes) = load_shader();

    let mut decompressed = Vec::new();
    let mut decoder = Decoder::new(compressed_bytes).expect("failed to create zstd decoder");
    decoder
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
    eprintln!("  cargo run --example include_minified_wgsl_compressed_zstd --features wgsl_minify");
}
