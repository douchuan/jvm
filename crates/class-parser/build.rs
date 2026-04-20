use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/fixtures/src");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile all .java files to OUT_DIR
    let java_files: Vec<_> = fs::read_dir(&src_dir)
        .expect("fixtures/src directory not found")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "java"))
        .map(|e| e.path())
        .collect();

    if java_files.is_empty() {
        return;
    }

    let mut cmd = Command::new("javac");
    cmd.arg("-d").arg(&out_dir);
    for f in &java_files {
        cmd.arg(f);
    }

    let status = cmd.status().expect("failed to run javac");
    assert!(status.success(), "javac failed");

    // Tell cargo to re-run if any .java file changes
    for f in &java_files {
        println!("cargo:rerun-if-changed={}", f.display());
    }

    // Export the output directory so tests can find .class files
    println!("cargo:rustc-env=FIXTURES_DIR={}", out_dir.display());
}
