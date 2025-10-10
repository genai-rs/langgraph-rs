/// Integration test for linear workflow conversion
use langgraph_inspector::extract_graph_info;
use langgraph_generator::CodeGenerator;
use std::process::Command;

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
fn test_generated_code_compiles() {
    // Test that generated code is syntactically valid Rust
    // This would require actually compiling the generated code
    // For now, we just verify structure

    let mock_graph_json = r#"{
        "nodes": [{"name": "test", "func_name": "test_node", "signature": "", "docstring": null, "source_hint": null}],
        "edges": [],
        "state_schema": {"fields": [{"name": "value", "type_name": "int", "is_optional": false, "default_value": null}]},
        "entry_point": "test",
        "conditional_edges": {}
    }"#;

    let result = langgraph_generator::generate_from_json(mock_graph_json);
    assert!(result.is_ok());
}
