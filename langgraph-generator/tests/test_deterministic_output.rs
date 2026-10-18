use std::fs;
use std::process::Command;

const OUTPUT_PATH_ENV: &str = "LANGGRAPH_GENERATOR_TEST_OUTPUT_PATH";

const UNORDERED_GRAPH: &str = r#"{
    "nodes": [
        {"name":"zeta","func_name":"zeta","signature":"","docstring":null,"source_hint":null},
        {"name":"alpha","func_name":"alpha","signature":"","docstring":null,"source_hint":null}
    ],
    "edges": [],
    "state_schema": {"fields": [
        {"name":"mapping","type_name":"dict[str, int]","is_optional":false,"default_value":null},
        {"name":"payload","type_name":"Any","is_optional":false,"default_value":null}
    ]},
    "entry_point":"zeta",
    "conditional_edges": {
        "zeta": {"condition_func":"route_zeta","branches":{"zulu":"END","alpha":"alpha","beta":"__end__"}},
        "alpha": {"condition_func":"route_alpha","branches":{"second":"END","first":"END"}}
    }
}"#;

#[test]
fn generation_output_writer_child() {
    let Ok(path) = std::env::var(OUTPUT_PATH_ENV) else {
        return;
    };

    let code = langgraph_generator::generate_from_json(UNORDERED_GRAPH).unwrap();
    fs::write(path, code).unwrap();
}

#[test]
fn generation_is_byte_identical_in_separate_processes() {
    let directory = std::env::temp_dir().join(format!(
        "langgraph-generator-deterministic-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();

    let test_binary = std::env::current_exe().unwrap();
    let outputs = [directory.join("first.rs"), directory.join("second.rs")];
    for output in &outputs {
        let result = Command::new(&test_binary)
            .args(["--exact", "generation_output_writer_child", "--nocapture"])
            .env(OUTPUT_PATH_ENV, output)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "child generation failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }

    assert_eq!(
        fs::read(&outputs[0]).unwrap(),
        fs::read(&outputs[1]).unwrap()
    );
    fs::remove_dir_all(directory).unwrap();
}
