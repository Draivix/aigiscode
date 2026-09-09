use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn source_files(directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            source_files(&entry.path(), files)?;
        } else {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing Cargo manifest directory")?);
    let mut files = vec![root.join("Cargo.toml"), root.join("build.rs")];
    source_files(&root.join("src"), &mut files)?;
    files.sort();
    let mut hash = DefaultHasher::new();
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        path.strip_prefix(&root)?.hash(&mut hash);
        fs::read(&path)?.hash(&mut hash);
    }
    // Cargo's resolved dependency set lives at the workspace root (or at the
    // package root for an installed binary crate). Include the actual lockfile.
    if let Some(workspace) = root
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
    {
        for name in ["Cargo.lock", "Cargo.toml"] {
            let path = workspace.join(name);
            println!("cargo:rerun-if-changed={}", path.display());
            name.hash(&mut hash);
            fs::read(path)?.hash(&mut hash);
        }
    } else {
        "no-lockfile".hash(&mut hash);
    }
    let mut configuration = env::vars()
        .filter(|(name, _)| {
            name.starts_with("CARGO_FEATURE_")
                || matches!(
                    name.as_str(),
                    "HOST"
                        | "TARGET"
                        | "PROFILE"
                        | "OPT_LEVEL"
                        | "DEBUG"
                        | "RUSTFLAGS"
                        | "CARGO_ENCODED_RUSTFLAGS"
                )
        })
        .collect::<Vec<_>>();
    configuration.sort();
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    for (name, value) in configuration {
        (name, value).hash(&mut hash);
    }
    let compiler = Command::new(env::var_os("RUSTC").ok_or("missing Rust compiler")?)
        .args(["--version", "--verbose"])
        .output()?;
    if !compiler.status.success() {
        return Err("could not identify Rust compiler".into());
    }
    compiler.stdout.hash(&mut hash);
    println!(
        "cargo:rustc-env=AIGISCODE_ENGINE_FINGERPRINT={:016x}",
        hash.finish()
    );
    Ok(())
}
