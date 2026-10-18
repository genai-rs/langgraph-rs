/// Integration test for linear workflow conversion

#[test]
fn test_linear_workflow_conversion() {
    // This test would require Python to be available
    // For now, we'll test with mock data

    let mock_graph_json = r#"{
        "nodes": [
            {
                "name": "a",
                "func_name": "node_a",
                "signature": "(state: SimpleState) -> SimpleState",
                "docstring": "First node in the chain.",
                "source_hint": "tests/fixtures/linear_workflow.py:15"
            },
            {
                "name": "b",
                "func_name": "node_b",
                "signature": "(state: SimpleState) -> SimpleState",
                "docstring": "Second node in the chain.",
                "source_hint": "tests/fixtures/linear_workflow.py:21"
            },
            {
                "name": "c",
                "func_name": "node_c",
                "signature": "(state: SimpleState) -> SimpleState",
                "docstring": "Final node in the chain.",
                "source_hint": "tests/fixtures/linear_workflow.py:27"
            }
        ],
        "edges": [
            {"from": "a", "to": "b", "condition": null},
            {"from": "b", "to": "c", "condition": null},
            {"from": "c", "to": "END", "condition": null}
        ],
        "state_schema": {
            "fields": [
                {"name": "counter", "type_name": "int", "is_optional": false, "default_value": null},
                {"name": "messages", "type_name": "list[str]", "is_optional": false, "default_value": null}
            ]
        },
        "entry_point": "a",
        "conditional_edges": {}
    }"#;

    let result = langgraph_generator::generate_from_json(mock_graph_json);
    assert!(result.is_ok(), "Code generation should succeed");

    let generated_code = result.unwrap();

    // Verify generated code contains expected elements
    assert!(generated_code.contains("struct GraphState"));
    assert!(generated_code.contains("counter: i64"));
    assert!(generated_code.contains("messages: Vec<String>"));
    assert!(generated_code.contains("async fn a_node"));
    assert!(generated_code.contains("async fn b_node"));
    assert!(generated_code.contains("async fn c_node"));
    assert!(generated_code.contains("execute_graph"));
}

#[test]
fn test_generated_code_has_expected_structure() {
    let mock_graph_json = r#"{
        "nodes": [{"name": "test", "func_name": "test_node", "signature": "", "docstring": null, "source_hint": null}],
        "edges": [],
        "state_schema": {"fields": [{"name": "value", "type_name": "int", "is_optional": false, "default_value": null}]},
        "entry_point": "test",
        "conditional_edges": {}
    }"#;

    let result = langgraph_generator::generate_from_json(mock_graph_json);
    let code = result.unwrap();
    assert!(code.contains("pub struct GraphState"));
    assert!(code.contains("value: i64"));
    assert!(code.contains("async fn test_node"));
}

#[test]
fn test_shuffled_nodes_follow_edge_order() {
    let graph = graph_json(
        &["c", "a", "b"],
        &[("a", "b"), ("b", "c"), ("c", "END")],
        "a",
    );
    let code = langgraph_generator::generate_from_json(&graph).unwrap();

    assert_eq!(execution_order(&code), ["a", "b", "c"]);
}

#[test]
fn test_linear_graph_rejects_missing_target() {
    let graph = graph_json(&["a"], &[("a", "missing")], "a");

    let error = langgraph_generator::generate_from_json(&graph).unwrap_err();
    assert!(error.to_string().contains("missing"));
    assert!(error
        .to_string()
        .contains("targets neither a graph node nor END"));
}

#[test]
fn test_linear_graph_rejects_multiple_successors() {
    let graph = graph_json(&["a", "b", "c"], &[("a", "b"), ("a", "c")], "a");

    let error = langgraph_generator::generate_from_json(&graph).unwrap_err();
    assert!(error.to_string().contains("multiple successors"));
    assert!(error.to_string().contains("a"));
}

#[test]
fn test_linear_graph_rejects_cycles() {
    let graph = graph_json(&["a", "b"], &[("a", "b"), ("b", "a")], "a");

    let error = langgraph_generator::generate_from_json(&graph).unwrap_err();
    assert!(error.to_string().contains("cycle"));
    assert!(error.to_string().contains("a"));
}

#[test]
fn test_linear_graph_rejects_unreachable_nodes() {
    let graph = graph_json(&["a", "b"], &[("a", "END")], "a");

    let error = langgraph_generator::generate_from_json(&graph).unwrap_err();
    assert!(error.to_string().contains("unreachable nodes"));
    assert!(error.to_string().contains("b"));
}

#[test]
fn test_linear_graph_rejects_duplicate_node_names() {
    let graph = graph_json(&["a", "a"], &[("a", "END")], "a");

    let error = langgraph_generator::generate_from_json(&graph).unwrap_err();
    assert!(error.to_string().contains("duplicate node name 'a'"));
}

#[test]
fn test_linear_graph_rejects_unknown_edge_sources() {
    let graph = graph_json(&["a"], &[("a", "END"), ("ghost", "missing")], "a");

    let error = langgraph_generator::generate_from_json(&graph).unwrap_err();
    assert!(error.to_string().contains("edge source 'ghost'"));
    assert!(error.to_string().contains("not a declared graph node"));
}

#[test]
fn test_linear_graph_rejects_missing_target_on_unreachable_edge() {
    let graph = graph_json(&["a", "b"], &[("a", "END"), ("b", "missing")], "a");

    let error = langgraph_generator::generate_from_json(&graph).unwrap_err();
    assert!(error.to_string().contains("edge 'b' -> 'missing'"));
    assert!(error
        .to_string()
        .contains("targets neither a graph node nor END"));
}

#[test]
fn test_linear_graph_rejects_condition_bearing_direct_edges() {
    let mut graph: serde_json::Value =
        serde_json::from_str(&graph_json(&["a", "b"], &[("a", "b"), ("b", "END")], "a")).unwrap();
    graph["edges"][0]["condition"] = serde_json::json!("false");

    let error = langgraph_generator::generate_from_json(&graph.to_string()).unwrap_err();
    assert!(error.to_string().contains("has a condition"));
    assert!(error.to_string().contains("conditional_edges metadata"));
}

#[test]
fn test_linear_graph_supports_start_sentinel_edge() {
    let graph = graph_json(&["a"], &[("__start__", "a"), ("a", "END")], "__start__");
    let code = langgraph_generator::generate_from_json(&graph).unwrap();

    assert_eq!(execution_order(&code), ["a"]);
}

fn graph_json(nodes: &[&str], edges: &[(&str, &str)], entry_point: &str) -> String {
    let nodes = nodes
        .iter()
        .map(|name| {
            serde_json::json!({
                "name": name,
                "func_name": format!("{name}_function"),
                "signature": "",
                "docstring": null,
                "source_hint": null
            })
        })
        .collect::<Vec<_>>();
    let edges = edges
        .iter()
        .map(|(from, to)| serde_json::json!({"from": from, "to": to, "condition": null}))
        .collect::<Vec<_>>();

    serde_json::json!({
        "nodes": nodes,
        "edges": edges,
        "state_schema": {"fields": []},
        "entry_point": entry_point,
        "conditional_edges": {}
    })
    .to_string()
}

fn execution_order(code: &str) -> Vec<String> {
    code.split("async fn execute_graph")
        .nth(1)
        .unwrap()
        .lines()
        .filter_map(|line| line.trim().strip_prefix("// Execute node: "))
        .map(str::to_owned)
        .collect()
}
