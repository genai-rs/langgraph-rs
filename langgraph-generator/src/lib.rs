use anyhow::Result;
use langgraph_inspector::type_mapping::{map_python_type, RustType};
use langgraph_inspector::{EdgeInfo, FieldInfo, GraphInfo, NodeInfo};
use std::collections::HashSet;

mod code_formatter;

pub struct CodeGenerator {
    graph_info: GraphInfo,
}

impl CodeGenerator {
    pub fn new(graph_info: GraphInfo) -> Self {
        Self { graph_info }
    }

    /// Generate complete Rust code from graph info
    pub fn generate_rust_code(&self) -> Result<String> {
        self.generate_rust_code_with_python(None)
    }

    /// Generate complete Rust code with original Python as reference
    pub fn generate_rust_code_with_python(&self, original_python: Option<&str>) -> Result<String> {
        let imports = self.get_required_imports();
        let state_struct = self.generate_state_struct()?;
        let node_functions = self.generate_node_functions()?;
        let graph_executor = self.generate_graph_executor()?;

        let code = code_formatter::format_generated_code(
            &imports,
            &state_struct,
            &node_functions,
            &graph_executor,
            original_python,
        );

        Ok(code)
    }

    /// Generate the state struct from schema
    fn generate_state_struct(&self) -> Result<String> {
        let fields: Vec<String> = self
            .graph_info
            .state_schema
            .fields
            .iter()
            .map(|field| self.field_to_rust(field))
            .collect();

        let struct_def = format!(
            r#"#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {{
{}
}}"#,
            fields.join(",\n")
        );

        Ok(struct_def)
    }

    /// Convert a field to Rust type
    fn field_to_rust(&self, field: &FieldInfo) -> String {
        let rust_type = map_python_type(&field.type_name);
        let type_str = rust_type.to_rust_string();

        if field.is_optional && !matches!(rust_type, RustType::Option(_)) {
            format!("    pub {}: Option<{}>", field.name, type_str)
        } else {
            format!("    pub {}: {}", field.name, type_str)
        }
    }

    /// Get all required imports for the generated code
    fn get_required_imports(&self) -> HashSet<String> {
        let mut imports = HashSet::new();

        // Collect imports from state fields
        for field in &self.graph_info.state_schema.fields {
            let rust_type = map_python_type(&field.type_name);
            for import in rust_type.required_imports() {
                imports.insert(import.to_string());
            }
        }

        imports
    }

    /// Generate node function stubs
    fn generate_node_functions(&self) -> Result<String> {
        let mut functions: Vec<String> = self
            .graph_info
            .nodes
            .iter()
            .map(|node| self.generate_node_function(node))
            .collect();

        // Add routing functions if we have conditional edges
        if !self.graph_info.conditional_edges.is_empty() {
            functions.push(self.generate_routing_functions());
        }

        Ok(functions.join("\n\n"))
    }

    /// Generate a single node function
    fn generate_node_function(&self, node: &NodeInfo) -> String {
        let docstring = node
            .docstring
            .as_deref()
            .unwrap_or("Generated node function");
        let doc = code_formatter::doc_comment(docstring);

        let source_comment = if let Some(hint) = &node.source_hint {
            format!("    // Original source: {}", hint)
        } else {
            "    // Source location not available".to_string()
        };

        format!(
            r#"{}
/// Original function: {}
/// Signature: {}
async fn {}_node(mut state: GraphState) -> Result<GraphState> {{
{}

    // TODO: Implement {} logic here
    // This is a stub - replace with actual implementation

    Ok(state)
}}"#,
            doc, node.func_name, node.signature, node.name, source_comment, node.name
        )
    }

    /// Generate the graph executor with conditional routing
    fn generate_graph_executor(&self) -> Result<String> {
        let entry_point = &self.graph_info.entry_point;
        let execution_logic = self.generate_execution_logic()?;

        let executor = format!(
            r#"/// Execute the compiled graph
async fn execute_graph(mut state: GraphState) -> Result<GraphState> {{
    tracing::info!("Starting graph execution from entry point: {entry}");

{logic}

    tracing::info!("Graph execution completed successfully");
    Ok(state)
}}"#,
            entry = entry_point,
            logic = execution_logic
        );

        Ok(executor)
    }

    /// Generate execution logic with conditional routing
    fn generate_execution_logic(&self) -> Result<String> {
        // Build execution flow considering edges and conditional edges
        let mut logic = String::new();

        // Check if we have conditional edges
        if !self.graph_info.conditional_edges.is_empty() {
            logic.push_str(&self.generate_conditional_execution()?);
        } else {
            logic.push_str(&self.generate_linear_execution());
        }

        Ok(logic)
    }

    /// Generate linear execution (no conditionals)
    fn generate_linear_execution(&self) -> String {
        self.graph_info
            .nodes
            .iter()
            .map(|node| {
                format!(
                    r#"    // Execute node: {}
    state = {}_node(state).await
        .context("Failed to execute node '{}')")?;"#,
                    node.name, node.name, node.name
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Generate conditional execution with routing
    fn generate_conditional_execution(&self) -> Result<String> {
        let mut execution = String::new();
        let entry = &self.graph_info.entry_point;

        execution.push_str(&format!(
            r#"    // Start with entry point
    let mut current_node = "{}";

    loop {{
        match current_node {{
"#,
            entry
        ));

        // Generate match arms for each node
        for node in &self.graph_info.nodes {
            execution.push_str(&format!(
                r#"            "{name}" => {{
                tracing::debug!("Executing node: {name}");
                state = {name}_node(state).await
                    .context("Failed to execute node '{name}')")?;

"#,
                name = node.name
            ));

            // Check if this node has conditional edges
            if let Some(cond_edge) = self.graph_info.conditional_edges.get(&node.name) {
                execution.push_str(&format!(
                    r#"                // Conditional routing from {}
                current_node = match {}_route(&state) {{
"#,
                    node.name, node.name
                ));

                for (condition, target) in &cond_edge.branches {
                    if target == "END" || target == "__end__" {
                        execution.push_str(&format!(
                            r#"                    "{}" => break,
"#,
                            condition
                        ));
                    } else {
                        execution.push_str(&format!(
                            r#"                    "{}" => "{}",
"#,
                            condition, target
                        ));
                    }
                }

                execution.push_str(
                    r#"                    _ => anyhow::bail!("Unknown routing condition"),
                };
            }
"#,
                );
            } else {
                // Check for direct edges
                if let Some(edge) = self.find_edge_from(&node.name) {
                    if edge.to == "END" || edge.to == "__end__" {
                        execution.push_str("                break;\n");
                    } else {
                        execution.push_str(&format!(
                            r#"                current_node = "{}";
"#,
                            edge.to
                        ));
                    }
                    execution.push_str("            }\n");
                } else {
                    execution.push_str("                break;\n");
                    execution.push_str("            }\n");
                }
            }
        }

        execution.push_str(
            r#"            "__end__" | "END" => break,
            _ => anyhow::bail!("Unknown node: {}", current_node),
        }
    }
"#,
        );

        Ok(execution)
    }

    /// Find edge from a given node
    fn find_edge_from(&self, from: &str) -> Option<&EdgeInfo> {
        self.graph_info.edges.iter().find(|e| e.from == from)
    }

    /// Generate routing functions for conditional edges
    fn generate_routing_functions(&self) -> String {
        let mut functions = Vec::new();

        for (node, cond_edge) in &self.graph_info.conditional_edges {
            let func = format!(
                r#"/// Routing function for node: {}
/// Original function: {}
fn {}_route(state: &GraphState) -> &'static str {{
    // TODO: Implement routing logic
    // This should return one of: {}
    "END"
}}"#,
                node,
                cond_edge.condition_func,
                node,
                cond_edge
                    .branches
                    .keys()
                    .map(|k| format!("\"{}\"", k))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            functions.push(func);
        }

        functions.join("\n\n")
    }
}

/// Generate Rust code from JSON graph info
pub fn generate_from_json(json: &str) -> Result<String> {
    let graph_info: GraphInfo = serde_json::from_str(json)?;
    let generator = CodeGenerator::new(graph_info);
    generator.generate_rust_code()
}
