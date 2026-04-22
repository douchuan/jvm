use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("java/src");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let java_files: Vec<_> = fs::read_dir(&src_dir)
        .expect("tests/java/src directory not found")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "java"))
        .map(|e| e.path())
        .collect();

    if java_files.is_empty() {
        return;
    }

    // Compile Java sources
    let mut cmd = Command::new("javac");
    cmd.arg("-d").arg(&out_dir);
    for f in &java_files {
        cmd.arg(f);
    }
    let status = cmd.status().expect("failed to run javac");
    assert!(status.success(), "javac failed");

    // Build list of testable classes (only files with main methods)
    let mut classes: Vec<String> = java_files
        .iter()
        .map(|p| p.file_stem().unwrap().to_str().unwrap().to_string())
        .collect();
    classes.sort();

    // Write class list for integration test
    let list_path = out_dir.join("classes.txt");
    fs::write(&list_path, classes.join("\n")).unwrap();

    println!("cargo:rustc-env=JAVA_TEST_DIR={}", out_dir.display());

    // Pass the jvm binary path
    let jvm_bin =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../target/debug/jvm");
    println!("cargo:rustc-env=JVM_BIN={}", jvm_bin.display());

    // Re-run if Java sources change
    for f in &java_files {
        println!("cargo:rerun-if-changed={}", f.display());
    }
}
