/// Integration test for conditional workflow conversion
use langgraph_generator::CodeGenerator;

#[test]
fn test_conditional_routing_generation() {
    let mock_graph_json = r#"{
        "nodes": [
            {"name": "start", "func_name": "start_node", "signature": "", "docstring": "Initialize", "source_hint": null},
            {"name": "high", "func_name": "process_high", "signature": "", "docstring": "Process high", "source_hint": null},
            {"name": "low", "func_name": "process_low", "signature": "", "docstring": "Process low", "source_hint": null},
            {"name": "end", "func_name": "end_node", "signature": "", "docstring": "Finalize", "source_hint": null}
        ],
        "edges": [],
        "state_schema": {
            "fields": [
                {"name": "value", "type_name": "int", "is_optional": false, "default_value": null},
                {"name": "path_taken", "type_name": "list[str]", "is_optional": false, "default_value": null}
            ]
        },
        "entry_point": "start",
        "conditional_edges": {
            "start": {
                "condition_func": "route_based_on_value",
                "branches": {
                    "high": "high",
                    "low": "low"
                }
            }
        }
    }"#;

    let result = langgraph_generator::generate_from_json(mock_graph_json);
    assert!(result.is_ok());

    let code = result.unwrap();

    // Verify conditional routing is generated
    assert!(code.contains("match current_node"));
    assert!(code.contains("start_route"));
    assert!(code.contains("route_based_on_value"));
}

#[test]
fn test_loop_detection() {
    let mock_graph_json = r#"{
        "nodes": [
            {"name": "loop_body", "func_name": "loop_body", "signature": "", "docstring": "Loop", "source_hint": null}
        ],
        "edges": [],
        "state_schema": {
            "fields": [{"name": "counter", "type_name": "int", "is_optional": false, "default_value": null}]
        },
        "entry_point": "loop_body",
        "conditional_edges": {
            "loop_body": {
                "condition_func": "should_continue",
                "branches": {
                    "loop_body": "loop_body",
                    "end": "END"
                }
            }
        }
    }"#;

    let result = langgraph_generator::generate_from_json(mock_graph_json);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("loop"));
    assert!(code.contains("break"));
}
