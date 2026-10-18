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
            .env_remove("LANGGRAPH_PYTHON")
            .args(args)
            .output()
            .unwrap()
    }

    fn run_without_python(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_langgraph-rs"))
            .current_dir(&self.0)
            .env_remove("RUST_LOG")
            .env_remove("LANGGRAPH_PYTHON")
            .env("PATH", "")
            .args(args)
            .output()
            .unwrap()
    }

    #[cfg(unix)]
    fn fake_python(&self, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let path = self.0.join("fake-python");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Ok(pid) = fs::read_to_string(self.0.join("timeout-child.pid")) {
            let _ = Command::new("kill").args(["-KILL", pid.trim()]).status();
        }
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
        let output = fixture.run_without_python(&args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn missing_input_and_directory_input_fail_without_creating_output() {
    let fixture = Fixture::new();
    for input in ["missing.py", "."] {
        assert_failure(
            fixture.run(&["convert", input, "--graph-object", "graph"]),
            input,
        );
        assert!(!fixture.0.join("generated").exists());
        assert_failure(
            fixture.run(&["inspect", input, "--graph-object", "graph"]),
            input,
        );
        assert_failure(
            fixture.run(&[
                "visualize",
                input,
                "--graph-object",
                "graph",
                "--output",
                "graph.mmd",
            ]),
            input,
        );
        assert!(!fixture.0.join("graph.mmd").exists());
    }
    assert_failure(fixture.run(&["convert"]), "PYTHON_FILE");
}

#[test]
fn extraction_fails_without_creating_or_overwriting_outputs() {
    let fixture = Fixture::new();
    let missing_python = fixture.0.join("missing-python");
    assert_failure(
        fixture.run(&[
            "convert",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            missing_python.to_str().unwrap(),
            "--with-tests",
        ]),
        "Cannot launch Python interpreter",
    );
    assert!(!fixture.0.join("generated").exists());
    fs::create_dir(fixture.0.join("generated")).unwrap();
    for file in ["lib.rs", "Cargo.toml", "tests.rs"] {
        fs::write(fixture.0.join("generated").join(file), "keep").unwrap();
    }
    assert_failure(
        fixture.run(&[
            "convert",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            missing_python.to_str().unwrap(),
            "--with-tests",
        ]),
        "Cannot launch Python interpreter",
    );
    for file in ["lib.rs", "Cargo.toml", "tests.rs"] {
        assert_eq!(
            fs::read_to_string(fixture.0.join("generated").join(file)).unwrap(),
            "keep"
        );
    }
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            missing_python.to_str().unwrap(),
        ]),
        "Cannot launch Python interpreter",
    );
    for format in ["mermaid", "dot"] {
        assert_failure(
            fixture.run(&[
                "visualize",
                "graph.py",
                "--graph-object",
                "graph",
                "--python",
                missing_python.to_str().unwrap(),
                "--format",
                format,
                "--output",
                "graph.out",
            ]),
            "Cannot launch Python interpreter",
        );
        assert!(!fixture.0.join("graph.out").exists());
    }
    fs::write(fixture.0.join("graph.out"), "keep").unwrap();
    assert_failure(
        fixture.run(&[
            "visualize",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            missing_python.to_str().unwrap(),
            "--output",
            "graph.out",
        ]),
        "Cannot launch Python interpreter",
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
            fixture.run(&[
                "inspect",
                "graph.py",
                "--graph-object",
                "graph",
                "--format",
                format,
            ]),
            "Unsupported inspect format",
        );
    }
    for format in ["svg", "unknown"] {
        assert_failure(
            fixture.run(&[
                "visualize",
                "graph.py",
                "--graph-object",
                "graph",
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
fn explicit_missing_interpreter_fails_without_fallback() {
    let fixture = Fixture::new();
    let missing_python = fixture.0.join("missing-python");
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            missing_python.to_str().unwrap(),
        ]),
        "Cannot launch Python interpreter",
    );
}

#[cfg(unix)]
#[test]
fn malformed_helper_output_fails_without_stdout() {
    let fixture = Fixture::new();
    let fake_python = fixture.fake_python("cat >/dev/null\nprintf 'not-json\\n'");
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            fake_python.to_str().unwrap(),
        ]),
        "malformed JSON",
    );
}

#[cfg(unix)]
#[test]
fn missing_helper_module_reports_python_diagnostic() {
    let fixture = Fixture::new();
    let fake_python = fixture.fake_python(
        "cat >/dev/null\nprintf \"ModuleNotFoundError: No module named 'langgraph_inspector'\\n\" >&2\nexit 1",
    );
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            fake_python.to_str().unwrap(),
        ]),
        "No module named 'langgraph_inspector'",
    );
}

#[cfg(unix)]
#[test]
fn stdout_limit_terminates_helper() {
    let fixture = Fixture::new();
    let fake_python = fixture.fake_python("cat >/dev/null\nwhile :; do printf '%4096s' x; done");
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            fake_python.to_str().unwrap(),
            "--timeout",
            "10s",
        ]),
        "stdout exceeded the 16 MiB",
    );
}

#[cfg(unix)]
#[test]
fn stderr_is_drained_and_only_its_tail_is_reported() {
    let fixture = Fixture::new();
    let fake_python = fixture.fake_python(
        "cat >/dev/null\ni=0\nwhile [ $i -lt 260 ]; do printf '%4096s' e >&2; i=$((i+1)); done\nprintf 'stderr-tail-marker\\n' >&2\nexit 1",
    );
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            fake_python.to_str().unwrap(),
        ]),
        "stderr-tail-marker",
    );
}

#[cfg(unix)]
#[test]
fn timeout_cancels_readers_when_descendant_holds_pipes() {
    use std::time::{Duration, Instant};

    let fixture = Fixture::new();
    let pid_path = fixture.0.join("timeout-child.pid");
    let fake_python = fixture.fake_python(&format!(
        "cat >/dev/null\nsleep 30 &\necho $! > '{}'\nwait",
        pid_path.display()
    ));
    let start = Instant::now();
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            fake_python.to_str().unwrap(),
            "--timeout",
            "1s",
        ]),
        "timed out",
    );
    assert!(start.elapsed() < Duration::from_secs(4));
    assert!(
        pid_path.is_file(),
        "test descendant was not recorded for cleanup"
    );
}

#[cfg(unix)]
#[test]
fn exited_helper_with_descendant_holding_pipes_times_out_without_panicking() {
    use std::time::{Duration, Instant};

    let fixture = Fixture::new();
    let pid_path = fixture.0.join("timeout-child.pid");
    let fake_python = fixture.fake_python(&format!(
        "cat >/dev/null\nsleep 30 &\necho $! > '{}'\nexit 0",
        pid_path.display()
    ));
    let start = Instant::now();
    assert_failure(
        fixture.run(&[
            "inspect",
            "graph.py",
            "--graph-object",
            "graph",
            "--python",
            fake_python.to_str().unwrap(),
            "--timeout",
            "1s",
        ]),
        "timed out while collecting output",
    );
    assert!(start.elapsed() < Duration::from_secs(4));
    assert!(
        pid_path.is_file(),
        "test descendant was not recorded for cleanup"
    );
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
