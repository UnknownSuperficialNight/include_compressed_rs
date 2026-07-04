#[cfg(feature = "wgsl_minify")]
use include_compressed::include_minified_wgsl;

#[cfg(feature = "wgsl_minify")]
fn main() {
    let original = include_str!("simple_shader.wgsl");
    let minified = include_minified_wgsl!("examples/simple_shader.wgsl");

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
