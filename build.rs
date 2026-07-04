use std::{env, path::PathBuf, process::Command};

/// Im pretty sure this does not work on windows if someone on windows wants to modify this build script to compile properly for both linux and windows be my guest
fn main() {
    // Only run if the wgsl_minify feature is enabled.
    if env::var("CARGO_FEATURE_WGSL_MINIFY").is_err() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let repo_url = "https://github.com/UnknownSuperficialNight/miniray-copy";
    let repo_dir = out_dir.join("miniray");

    // Candidate locations for the built static library.
    let candidate_build = repo_dir.join("build").join("libminiray.a");
    let candidate_target = repo_dir.join("target").join("libminiray.a");

    // Fast path: If the library already exists, emit link directives and exit.
    if let Some(lib_path) = first_existing(&[&candidate_build, &candidate_target]) {
        emit_link_directives(&out_dir, lib_path);
        return;
    }

    // 4. Clone the repo only if Makefile is missing. No cargo:warning on success.
    if !repo_dir.join("Makefile").exists() {
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                repo_url,
                repo_dir.to_str().unwrap(),
            ])
            .status()
            .expect("build.rs: failed to spawn `git clone`");
        assert!(status.success(), "build.rs: `git clone` of miniray failed");
    }

    // 5. Build the C library. Only emit cargo:warning if make fails.
    let make_output = Command::new("make")
        .arg("lib")
        .current_dir(&repo_dir)
        .output()
        .expect("build.rs: failed to spawn `make`");

    if !make_output.status.success() {
        println!(
            "cargo:warning=make lib stdout:\n{}",
            String::from_utf8_lossy(&make_output.stdout)
        );
        println!(
            "cargo:warning=make lib stderr:\n{}",
            String::from_utf8_lossy(&make_output.stderr)
        );
        panic!("build.rs: `make lib` failed — see warnings above for details");
    }

    // 6. Locate the built library and emit link directives.
    let lib_path = first_existing(&[&candidate_build, &candidate_target]).expect(
        "build.rs: libminiray.a not found after `make lib` in either `build/` or `target/`",
    );

    emit_link_directives(&out_dir, lib_path);
}

// Helper: Return the first path in `candidates` that exists.
fn first_existing<'a>(candidates: &[&'a PathBuf]) -> Option<&'a PathBuf> {
    candidates.iter().copied().find(|p| p.exists())
}

// Helper: Emit Cargo linker directives, with duplicate-symbol guard.
fn emit_link_directives(out_dir: &PathBuf, lib_path: &PathBuf) {
    let lib_dir = lib_path.parent().expect("lib_path has no parent directory");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // 7. If libinclude_compressed.so is present, skip explicit static linking.
    if libinclude_compressed_is_available(out_dir) {
        // Assume symbols are already provided; do not link miniray.a again.
        return;
    }

    // Standard case: link the static miniray archive directly.
    println!("cargo:rustc-link-lib=static=miniray");
}

// Returns true if libinclude_compressed.so is found in OUT_DIR or library paths.
fn libinclude_compressed_is_available(out_dir: &PathBuf) -> bool {
    const TARGET: &str = "libinclude_compressed.so";
    if out_dir.join(TARGET).exists() {
        return true;
    }
    for var in &["LIBRARY_PATH", "LD_LIBRARY_PATH"] {
        if let Ok(paths) = env::var(var) {
            for dir in env::split_paths(&paths) {
                if dir.join(TARGET).exists() {
                    return true;
                }
            }
        }
    }
    false
}
