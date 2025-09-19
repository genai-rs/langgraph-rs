use anyhow::Result;
use langgraph_inspector::{FieldInfo, GraphInfo, NodeInfo};
use quote::{format_ident, quote};
use std::collections::HashMap;

pub struct CodeGenerator {
    graph_info: GraphInfo,
}

impl CodeGenerator {
    pub fn new(graph_info: GraphInfo) -> Self {
        Self { graph_info }
    }

    /// Generate complete Rust code from graph info
    pub fn generate_rust_code(&self) -> Result<String> {
        let state_struct = self.generate_state_struct()?;
        let node_functions = self.generate_node_functions()?;
        let graph_executor = self.generate_graph_executor()?;

        let code = format!(
            r#"use serde::{{Deserialize, Serialize}};
use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;

// Generated State Structure
{}

// Generated Node Functions
{}

// Graph Executor
{}

// Main entry point
pub async fn run_graph(initial_state: GraphState) -> Result<GraphState> {{
    execute_graph(initial_state).await
}}
"#,
            state_struct, node_functions, graph_executor
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
        let rust_type = self.python_type_to_rust(&field.type_name);

        if field.is_optional {
            format!("    pub {}: Option<{}>", field.name, rust_type)
        } else {
            format!("    pub {}: {}", field.name, rust_type)
        }
    }

    /// Map Python types to Rust types
    fn python_type_to_rust(&self, py_type: &str) -> String {
        match py_type {
            "str" => "String".to_string(),
            "int" => "i64".to_string(),
            "float" => "f64".to_string(),
            "bool" => "bool".to_string(),
            "list" | "List" => "Vec<serde_json::Value>".to_string(),
            "dict" | "Dict" => "HashMap<String, serde_json::Value>".to_string(),
            _ => "serde_json::Value".to_string(),
        }
    }

    /// Generate node function stubs
    fn generate_node_functions(&self) -> Result<String> {
        let functions: Vec<String> = self
            .graph_info
            .nodes
            .iter()
            .map(|node| self.generate_node_function(node))
            .collect();

        Ok(functions.join("\n\n"))
    }

    /// Generate a single node function
    fn generate_node_function(&self, node: &NodeInfo) -> String {
        let func_name = format_ident!("{}_node", node.name);
        let docstring = node.docstring.as_deref().unwrap_or("Generated node function");

        format!(
            r#"/// {}
async fn {}(state: GraphState) -> Result<GraphState> {{
    // TODO: Implement {} logic
    // Original function: {}
    {}

    Ok(state)
}}"#,
            docstring,
            func_name,
            node.name,
            node.func_name,
            node.source_hint.as_deref().unwrap_or("// Source not available")
        )
    }

    /// Generate the graph executor
    fn generate_graph_executor(&self) -> Result<String> {
        let entry_point = &self.graph_info.entry_point;

        // Simple linear execution for now
        // TODO: Handle conditional edges and complex routing
        let executor = format!(
            r#"async fn execute_graph(mut state: GraphState) -> Result<GraphState> {{
    // Starting from entry point: {}

    // Execute nodes in sequence
    {}

    Ok(state)
}}"#,
            entry_point,
            self.generate_execution_sequence()
        );

        Ok(executor)
    }

    /// Generate execution sequence
    fn generate_execution_sequence(&self) -> String {
        self.graph_info
            .nodes
            .iter()
            .map(|node| format!("    state = {}_node(state).await?;", node.name))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Generate Rust code from JSON graph info
pub fn generate_from_json(json: &str) -> Result<String> {
    let graph_info: GraphInfo = serde_json::from_str(json)?;
    let generator = CodeGenerator::new(graph_info);
    generator.generate_rust_code()
}