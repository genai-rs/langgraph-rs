use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "langgraph-cli-contract-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(
            path.join("graph.py"),
            "# Input must never produce a canned graph\n",
        )
        .unwrap();
        fs::write(path.join("lib.rs"), "// Rust input\n").unwrap();
        fs::write(path.join("data.json"), "{}").unwrap();
        Self(path)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_langgraph-rs"))
            .current_dir(&self.0)
            .env_remove("RUST_LOG")
            .args(args)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn assert_failure(output: Output, message: &str) {
    assert!(!output.status.success(), "unexpected success: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(message),
        "expected {message:?} in stderr: {stderr}"
    );
    assert!(output.stdout.is_empty(), "unexpected stdout: {output:?}");
}

#[test]
fn help_and_version_succeed() {
    let fixture = Fixture::new();
    for args in [
        vec!["--help"],
        vec!["--version"],
        vec!["convert", "--help"],
        vec!["inspect", "--help"],
        vec!["validate", "--help"],
        vec!["visualize", "--help"],
    ] {
        let output = fixture.run(&args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn missing_input_and_directory_input_fail_without_creating_output() {
    let fixture = Fixture::new();
    for input in ["missing.py", "."] {
        assert_failure(fixture.run(&["convert", input]), input);
        assert!(!fixture.0.join("generated").exists());
        assert_failure(fixture.run(&["inspect", input]), input);
        assert_failure(
            fixture.run(&["visualize", input, "--output", "graph.mmd"]),
            input,
        );
        assert!(!fixture.0.join("graph.mmd").exists());
    }
    assert_failure(fixture.run(&["convert"]), "PYTHON_FILE");
}

#[test]
fn extraction_fails_without_creating_or_overwriting_outputs() {
    let fixture = Fixture::new();
    assert_failure(
        fixture.run(&["convert", "graph.py", "--with-tests"]),
        "extraction is not implemented",
    );
    assert!(!fixture.0.join("generated").exists());
    fs::create_dir(fixture.0.join("generated")).unwrap();
    for file in ["lib.rs", "Cargo.toml", "tests.rs"] {
        fs::write(fixture.0.join("generated").join(file), "keep").unwrap();
    }
    assert_failure(
        fixture.run(&["convert", "graph.py", "--with-tests"]),
        "extraction is not implemented",
    );
    for file in ["lib.rs", "Cargo.toml", "tests.rs"] {
        assert_eq!(
            fs::read_to_string(fixture.0.join("generated").join(file)).unwrap(),
            "keep"
        );
    }
    assert_failure(
        fixture.run(&["inspect", "graph.py"]),
        "extraction is not implemented",
    );
    for format in ["mermaid", "dot"] {
        assert_failure(
            fixture.run(&[
                "visualize",
                "graph.py",
                "--format",
                format,
                "--output",
                "graph.out",
            ]),
            "extraction is not implemented",
        );
        assert!(!fixture.0.join("graph.out").exists());
    }
    fs::write(fixture.0.join("graph.out"), "keep").unwrap();
    assert_failure(
        fixture.run(&["visualize", "graph.py", "--output", "graph.out"]),
        "extraction is not implemented",
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("graph.out")).unwrap(),
        "keep"
    );
}

#[test]
fn unsupported_formats_fail_before_extraction() {
    let fixture = Fixture::new();
    for format in ["yaml", "toml", "unknown"] {
        assert_failure(
            fixture.run(&["inspect", "graph.py", "--format", format]),
            "Unsupported inspect format",
        );
    }
    for format in ["svg", "unknown"] {
        assert_failure(
            fixture.run(&[
                "visualize",
                "graph.py",
                "--format",
                format,
                "--output",
                "graph.out",
            ]),
            "Unsupported visualization format",
        );
        assert!(!fixture.0.join("graph.out").exists());
    }
}

#[test]
fn validation_checks_paths_and_reports_unimplemented() {
    let fixture = Fixture::new();
    assert_failure(
        fixture.run(&["validate", "--python", "missing.py", "--rust", "lib.rs"]),
        "missing.py",
    );
    assert_failure(
        fixture.run(&["validate", "--python", "graph.py", "--rust", "missing.rs"]),
        "missing.rs",
    );
    assert_failure(
        fixture.run(&[
            "validate",
            "--python",
            "graph.py",
            "--rust",
            "lib.rs",
            "--test-data",
            "missing.json",
        ]),
        "missing.json",
    );
    assert_failure(
        fixture.run(&["validate", "--python", "graph.py", "--rust", "lib.rs"]),
        "Validation is not implemented",
    );
    assert_failure(
        fixture.run(&["validate", "--python", "graph.py", "--rust", "."]),
        "Validation is not implemented",
    );
    assert_failure(
        fixture.run(&[
            "validate",
            "--python",
            "graph.py",
            "--rust",
            "lib.rs",
            "--test-data",
            "data.json",
        ]),
        "Validation is not implemented",
    );
}
