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
    "nodes": [
        {"name":"finish","func_name":"finish","signature":"(state) -> state","docstring":null,"source_hint":null},
        {"name":"step","func_name":"step","signature":"(state) -> state","docstring":null,"source_hint":null}
    ],
    "edges": [
        {"from":"step","to":"finish","condition":null},
        {"from":"finish","to":"END","condition":null}
    ],
    "state_schema": {"fields": [
        {"name":"counter","type_name":"int","is_optional":false,"default_value":null},
        {"name":"messages","type_name":"list[str]","is_optional":false,"default_value":null},
        {"name":"optional_value","type_name":"Optional[Any]","is_optional":true,"default_value":null},
        {"name":"attributes","type_name":"dict[str, Any]","is_optional":false,"default_value":null}
    ]},
    "entry_point":"step",
    "conditional_edges":{}
}"#;

#[test]
fn generated_names_compile_and_regeneration_removes_only_owned_tests() {
    let temp = TempDir::new();
    let rust = langgraph_generator::generate_from_json(LINEAR_GRAPH).unwrap();
    let cases = [
        ("9 My Graph!.py", "generated_9_my_graph"),
        ("test.py", "generated_test"),
        ("std.py", "generated_std"),
        ("core.py", "generated_core"),
        ("anyhow.py", "generated_anyhow"),
        ("serde.py", "generated_serde"),
        ("serde_json.py", "generated_serde_json"),
        ("tracing.py", "generated_tracing"),
        ("tokio.py", "generated_tokio"),
        ("two steps!.py", "two_steps"),
    ];

    let mut first_package = None;
    for (index, (input_name, expected_name)) in cases.iter().enumerate() {
        let package = temp.0.join(format!("package-{index}"));
        let name =
            write_generated_package(&package, Path::new(input_name), &rust, LINEAR_GRAPH, true)
                .unwrap();
        assert_eq!(name, *expected_name);
        let manifest = fs::read_to_string(package.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("anyhow = \"1\""));
        assert!(manifest.contains("serde = { version = \"1\", features = [\"derive\"] }"));
        assert!(manifest.contains("serde_json = \"1\""));
        assert!(manifest.contains("tracing = \"0.1\""));
        assert!(manifest.contains("tokio = { version = \"1\", features = [\"macros\", \"rt\"] }"));
        assert!(!manifest.contains("async-trait ="));
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
            "metadata failed for {input_name}: {}",
            String::from_utf8_lossy(&metadata.stderr)
        );
        let metadata: Value = serde_json::from_slice(&metadata.stdout).unwrap();
        let package_metadata = &metadata["packages"][0];
        assert_eq!(package_metadata["name"], name);
        let targets = package_metadata["targets"].as_array().unwrap();
        assert!(targets.iter().any(|target| target["kind"][0] == "lib"));
        assert!(targets.iter().any(|target| target["kind"][0] == "test"));

        run_cargo_test(&temp.0, &package, "generated_graph_runs ... ok");
        if index == 0 {
            first_package = Some(package);
        }
    }

    let package = first_package.unwrap();
    let tests_dir = package.join("tests");
    fs::write(
        tests_dir.join("custom.rs"),
        "#[test] fn user_test_survives() { assert_eq!(2 + 2, 4); }\n",
    )
    .unwrap();
    write_generated_package(
        &package,
        Path::new("9 My Graph!.py"),
        &rust,
        LINEAR_GRAPH,
        false,
    )
    .unwrap();
    assert!(!tests_dir.join("generated_graph.rs").exists());
    assert!(tests_dir.join("custom.rs").is_file());
    let manifest = fs::read_to_string(package.join("Cargo.toml")).unwrap();
    assert!(!manifest.contains("tokio ="));
    run_cargo_test(&temp.0, &package, "user_test_survives ... ok");

    let user_owned_test = "#[test] fn user_owned_test() { assert!(true); }\n";
    fs::write(tests_dir.join("generated_graph.rs"), user_owned_test).unwrap();
    write_generated_package(
        &package,
        Path::new("9 My Graph!.py"),
        &rust,
        LINEAR_GRAPH,
        false,
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(tests_dir.join("generated_graph.rs")).unwrap(),
        user_owned_test
    );
    run_cargo_test(&temp.0, &package, "user_owned_test ... ok");
}

fn run_cargo_test(directory: &Path, package: &Path, expected_test: &str) {
    let target_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/generated-package-fixture");
    let tests = Command::new("cargo")
        .args(["test", "--manifest-path"])
        .arg(package.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", target_dir)
        .env("CARGO_BUILD_JOBS", "2")
        .current_dir(directory)
        .output()
        .unwrap();
    assert!(
        tests.status.success(),
        "cargo test failed for {}\nstdout: {}\nstderr: {}",
        package.display(),
        String::from_utf8_lossy(&tests.stdout),
        String::from_utf8_lossy(&tests.stderr)
    );
    let stdout = String::from_utf8_lossy(&tests.stdout);
    assert!(stdout.contains(expected_test), "{stdout}");
}
