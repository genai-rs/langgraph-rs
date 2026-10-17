use langgraph_cli::package_writer::write_generated_package;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "langgraph-generated-package-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

const LINEAR_GRAPH: &str = r#"{
    "nodes": [{"name":"step","func_name":"step","signature":"(state) -> state","docstring":null,"source_hint":null}],
    "edges": [],
    "state_schema": {"fields": [
        {"name":"counter","type_name":"int","is_optional":false,"default_value":null},
        {"name":"messages","type_name":"list[str]","is_optional":false,"default_value":null}
    ]},
    "entry_point":"step",
    "conditional_edges":{}
}"#;

#[test]
fn generated_package_is_discoverable_and_its_generated_test_runs() {
    let temp = TempDir::new();
    let package = temp.0.join("package");
    let rust = langgraph_generator::generate_from_json(LINEAR_GRAPH).unwrap();
    let name = write_generated_package(
        &package,
        Path::new("9 My Graph!.py"),
        &rust,
        LINEAR_GRAPH,
        true,
    )
    .unwrap();

    assert_eq!(name, "generated_9_my_graph");
    let manifest = fs::read_to_string(package.join("Cargo.toml")).unwrap();
    assert!(manifest.contains("tracing = \"0.1\""));
    assert!(manifest.contains("tokio = { version = \"1\", features = [\"macros\", \"rt\"] }"));
    assert!(!manifest.contains("async-trait ="));
    assert!(!manifest.contains("serde_json ="));
    assert!(!manifest.contains("langgraph-runtime ="));
    assert!(package.join("src/lib.rs").is_file());
    assert!(package.join("tests/generated_graph.rs").is_file());

    let metadata = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(package.join("Cargo.toml"))
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let metadata: Value = serde_json::from_slice(&metadata.stdout).unwrap();
    let package_metadata = &metadata["packages"][0];
    assert_eq!(package_metadata["name"], name);
    let targets = package_metadata["targets"].as_array().unwrap();
    assert!(targets.iter().any(|target| target["kind"][0] == "lib"));
    assert!(targets.iter().any(|target| target["kind"][0] == "test"));

    let target_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/generated-package-fixture");
    let tests = Command::new("cargo")
        .args(["test", "--manifest-path"])
        .arg(package.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", target_dir)
        .env("CARGO_BUILD_JOBS", "2")
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(
        tests.status.success(),
        "cargo test failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&tests.stdout),
        String::from_utf8_lossy(&tests.stderr)
    );
    let stdout = String::from_utf8_lossy(&tests.stdout);
    assert!(stdout.contains("generated_graph_runs ... ok"), "{stdout}");
}
