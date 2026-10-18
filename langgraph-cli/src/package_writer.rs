use anyhow::{Context, Result};
use langgraph_inspector::type_mapping::{map_python_type, RustType};
use langgraph_inspector::GraphInfo;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Write generated Rust code as a standalone Cargo package.
pub fn write_generated_package(
    output: &Path,
    input: &Path,
    rust_code: &str,
    graph_json: &str,
    with_tests: bool,
) -> Result<String> {
    let graph: GraphInfo = serde_json::from_str(graph_json).context("Invalid graph metadata")?;
    let package_name = package_name(input);
    let test_code = with_tests.then(|| generate_test_code(&graph, &package_name));
    let mut dependencies = HashSet::from([("anyhow", "1"), ("serde", "1"), ("tracing", "0.1")]);
    if graph
        .state_schema
        .fields
        .iter()
        .any(|field| uses_json_value(&map_python_type(&field.type_name)))
    {
        dependencies.insert(("serde_json", "1"));
    }

    fs::create_dir_all(output.join("src")).with_context(|| {
        format!(
            "Cannot create generated source directory: {}",
            output.display()
        )
    })?;
    fs::write(output.join("src/lib.rs"), rust_code)
        .with_context(|| format!("Cannot write generated library: {}", output.display()))?;
    fs::write(
        output.join("Cargo.toml"),
        manifest(&package_name, &dependencies, with_tests),
    )
    .with_context(|| format!("Cannot write generated manifest: {}", output.display()))?;

    let generated_test = output.join("tests/generated_graph.rs");
    if let Some(test_code) = test_code {
        let tests = output.join("tests");
        fs::create_dir_all(&tests).with_context(|| {
            format!(
                "Cannot create generated tests directory: {}",
                tests.display()
            )
        })?;
        fs::write(&generated_test, test_code).with_context(|| {
            format!(
                "Cannot write generated integration test: {}",
                tests.display()
            )
        })?;
    } else {
        remove_owned_generated_test(&generated_test)?;
    }

    Ok(package_name)
}

fn package_name(input: &Path) -> String {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");
    let mut name = String::new();
    let mut previous_underscore = false;
    for character in stem.chars() {
        let character = if character.is_ascii_alphanumeric() || character == '_' {
            character.to_ascii_lowercase()
        } else {
            '_'
        };
        if character == '_' {
            if !previous_underscore {
                name.push(character);
            }
        } else {
            name.push(character);
        }
        previous_underscore = character == '_';
    }
    let name = name.trim_matches('_').to_owned();
    const RUST_CRATE_NAME_COLLISIONS: &[&str] = &[
        "as",
        "async",
        "await",
        "break",
        "const",
        "continue",
        "crate",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "Self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
        "abstract",
        "become",
        "box",
        "do",
        "final",
        "macro",
        "override",
        "priv",
        "typeof",
        "unsized",
        "virtual",
        "yield",
        "try",
        "gen",
        "std",
        "core",
        "alloc",
        "proc_macro",
        "test",
        "anyhow",
        "serde",
        "serde_json",
        "tracing",
        "tokio",
    ];
    if name.is_empty()
        || name.as_bytes()[0].is_ascii_digit()
        || RUST_CRATE_NAME_COLLISIONS.contains(&name.as_str())
    {
        format!(
            "generated_{}",
            if name.is_empty() { "graph" } else { &name }
        )
    } else {
        name
    }
}

fn uses_json_value(ty: &RustType) -> bool {
    match ty {
        RustType::JsonValue => true,
        RustType::Vec(inner) | RustType::Option(inner) => uses_json_value(inner),
        RustType::HashMap(key, value) => uses_json_value(key) || uses_json_value(value),
        _ => false,
    }
}

fn manifest(name: &str, dependencies: &HashSet<(&str, &str)>, with_tests: bool) -> String {
    let mut manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n# Generated graph code is self-contained and does not depend on langgraph-runtime.\n[dependencies]\n"
    );
    let mut dependencies: Vec<_> = dependencies.iter().copied().collect();
    dependencies.sort_unstable_by_key(|(name, _)| *name);
    for (name, version) in dependencies {
        if name == "serde" {
            manifest.push_str(&format!(
                "serde = {{ version = \"{version}\", features = [\"derive\"] }}\n"
            ));
        } else {
            manifest.push_str(&format!("{name} = \"{version}\"\n"));
        }
    }
    if with_tests {
        manifest.push_str(
            "\n[dev-dependencies]\ntokio = { version = \"1\", features = [\"macros\", \"rt\"] }\n",
        );
    }
    manifest
}

const GENERATED_TEST_MARKER: &str = "// Generated by langgraph-rs. Do not edit.";

fn remove_owned_generated_test(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Cannot inspect generated test: {}", path.display()))?;
            if contents.starts_with(GENERATED_TEST_MARKER) {
                fs::remove_file(path).with_context(|| {
                    format!("Cannot remove stale generated test: {}", path.display())
                })?;
            }
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Cannot inspect generated test: {}", path.display()));
        }
    }
    Ok(())
}

fn generate_test_code(graph: &GraphInfo, package_name: &str) -> String {
    let fields = graph
        .state_schema
        .fields
        .iter()
        .map(|field| format!("        {}: Default::default(),", field.name))
        .collect::<Vec<_>>()
        .join("\n");
    let state = if fields.is_empty() {
        "GraphState {}".to_owned()
    } else {
        format!("GraphState {{\n{fields}\n    }}")
    };
    let first_node = first_executed_node(graph);
    format!(
        "{marker}\nuse {crate_name}::{{run_graph, GraphState}};\n\n#[tokio::test]\nasync fn generated_graph_reports_unimplemented_node() {{\n    let error = run_graph({state}).await.unwrap_err();\n    assert!(format!(\"{{error:#}}\").contains(\"Node '{first_node}' is not implemented\"), \"unexpected scaffold error: {{error:#}}\");\n}}\n",
        marker = GENERATED_TEST_MARKER,
        crate_name = package_name,
        first_node = first_node
    )
}

fn first_executed_node(graph: &GraphInfo) -> &str {
    if graph
        .nodes
        .iter()
        .any(|node| node.name == graph.entry_point)
    {
        return &graph.entry_point;
    }

    graph
        .edges
        .iter()
        .find(|edge| {
            edge.from == graph.entry_point && graph.nodes.iter().any(|node| node.name == edge.to)
        })
        .map(|edge| edge.to.as_str())
        .unwrap_or(&graph.entry_point)
}

#[cfg(test)]
mod tests {
    use super::package_name;
    use std::path::Path;

    #[test]
    fn package_names_are_valid_crate_identifiers() {
        assert_eq!(
            package_name(Path::new("9 My Graph!.py")),
            "generated_9_my_graph"
        );
        for name in [
            "self.py",
            "std.py",
            "test.py",
            "core.py",
            "anyhow.py",
            "serde.py",
            "serde_json.py",
            "tracing.py",
            "tokio.py",
        ] {
            assert_eq!(
                package_name(Path::new(name)),
                format!("generated_{}", name.trim_end_matches(".py"))
            );
        }
        assert_eq!(package_name(Path::new("!!!.py")), "generated_graph");
    }
}
