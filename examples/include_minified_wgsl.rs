#[cfg(feature = "wgsl_minify")]
use include_compressed::include_minified_wgsl;

fn load_shader() -> (&'static str, &'static str) {
    #[cfg(feature = "large_shader")]
    {
        let original = include_str!("large_shader.wgsl");

        let compressed_bytes = include_minified_wgsl!("examples/large_shader.wgsl");

        (original, compressed_bytes)
    }

    #[cfg(not(feature = "large_shader"))]
    {
        let original = include_str!("../examples/simple_shader.wgsl");

        let compressed_bytes = include_minified_wgsl!("examples/simple_shader.wgsl");

        (original, compressed_bytes)
    }
}

#[cfg(feature = "wgsl_minify")]
fn main() {
    let (original, minified) = load_shader();

    println!("Original WGSL:");
    println!("{original}");
    println!();

    println!("Minified WGSL:");
    println!("{minified}");
    println!();

    println!("Original size: {} bytes", original.len());
    println!("Minified size: {} bytes", minified.len());
    println!(
        "Size reduction: {:.2}%",
        100.0 - (minified.len() as f64 / original.len() as f64 * 100.0)
    );
}

// Fallback for when wgsl_minify feature is not enabled
#[cfg(not(feature = "wgsl_minify"))]
fn main() {
    eprintln!("==============================================");
    eprintln!("  ERROR: wgsl_minify feature is not enabled   ");
    eprintln!("==============================================");
    eprintln!();
    eprintln!("This example requires the `wgsl_minify` feature.");
    eprintln!("Run:");
    eprintln!("  cargo run --example include_minified_wgsl --features wgsl_minify");
}
