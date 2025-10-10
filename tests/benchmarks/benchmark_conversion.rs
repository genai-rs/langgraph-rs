/// Benchmarks for code generation performance
#[cfg(test)]
mod benchmarks {
    use langgraph_generator::generate_from_json;

    fn generate_large_graph_json(num_nodes: usize) -> String {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for i in 0..num_nodes {
            nodes.push(format!(
                r#"{{"name": "node_{}", "func_name": "node_{}", "signature": "", "docstring": null, "source_hint": null}}"#,
                i, i
            ));

            if i > 0 {
                edges.push(format!(
                    r#"{{"from": "node_{}", "to": "node_{}", "condition": null}}"#,
                    i - 1, i
                ));
            }
        }

        format!(
            r#"{{
                "nodes": [{}],
                "edges": [{}],
                "state_schema": {{"fields": [{{"name": "value", "type_name": "int", "is_optional": false, "default_value": null}}]}},
                "entry_point": "node_0",
                "conditional_edges": {{}}
            }}"#,
            nodes.join(","),
            edges.join(",")
        )
    }

    #[test]
    fn bench_small_graph() {
        let json = generate_large_graph_json(5);
        let start = std::time::Instant::now();

        for _ in 0..100 {
            let _ = generate_from_json(&json);
        }

        let duration = start.elapsed();
        println!("Small graph (5 nodes) x100: {:?}", duration);

        // Should be fast
        assert!(duration.as_millis() < 1000, "Generation too slow");
    }

    #[test]
    fn bench_medium_graph() {
        let json = generate_large_graph_json(20);
        let start = std::time::Instant::now();

        for _ in 0..100 {
            let _ = generate_from_json(&json);
        }

        let duration = start.elapsed();
        println!("Medium graph (20 nodes) x100: {:?}", duration);

        assert!(duration.as_secs() < 5, "Generation too slow");
    }

    #[test]
    fn bench_large_graph() {
        let json = generate_large_graph_json(100);
        let start = std::time::Instant::now();

        for _ in 0..10 {
            let _ = generate_from_json(&json);
        }

        let duration = start.elapsed();
        println!("Large graph (100 nodes) x10: {:?}", duration);

        assert!(duration.as_secs() < 10, "Generation too slow");
    }
}
