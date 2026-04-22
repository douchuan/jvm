use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let class_list = env!("JAVA_TEST_DIR");
    let jvm_bin = env!("JVM_BIN");
    let classes = fs::read_to_string(PathBuf::from(class_list).join("classes.txt"))
        .expect("failed to read classes.txt");

    let classes: Vec<&str> = classes.lines().filter(|s| !s.is_empty()).collect();

    let mut pass = 0;
    let mut fail = 0;

    println!("Running {} Java tests against the JVM...\n", classes.len());

    for class in &classes {
        let output = Command::new(jvm_bin)
            .arg("--cp")
            .arg(class_list)
            .arg(class)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                println!("  java::{} ... ok", class);
                pass += 1;
            }
            Ok(out) => {
                println!("  java::{} ... FAILED", class);
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stderr.lines().take(3) {
                    println!("    {}", line);
                }
                fail += 1;
            }
            Err(e) => {
                println!("  java::{} ... FAILED ({})", class, e);
                fail += 1;
            }
        }
    }

    println!("\n{} passed, {} failed\n", pass, fail);

    if fail > 0 {
        std::process::exit(1);
    }
}
