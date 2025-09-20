use anyhow::Result;
use langgraph_inspector::{FieldInfo, GraphInfo, NodeInfo};
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};

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

        // Generate field initializers for new() constructor
        let field_initializers: Vec<String> = self
            .graph_info
            .state_schema
            .fields
            .iter()
            .map(|field| {
                let name = sanitize_identifier(&field.name);
                if field.is_optional {
                    format!("            {}: None", name)
                } else {
                    match field.type_name.as_str() {
                        "list" => format!("            {}: Vec::new()", name),
                        "dict" => format!("            {}: HashMap::new()", name),
                        "str" => format!("            {}: String::new()", name),
                        "int" => format!("            {}: 0", name),
                        "float" => format!("            {}: 0.0", name),
                        "bool" => format!("            {}: false", name),
                        _ => format!("            {}: Default::default()", name),
                    }
                }
            })
            .collect();

        let struct_def = format!(
            r#"#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {{
{}
}}

impl GraphState {{
    /// Create a new instance with default values
    pub fn new() -> Self {{
        Self {{
{}
        }}
    }}
}}"#,
            fields.join(",\n"),
            field_initializers.join(",\n")
        );

        Ok(struct_def)
    }

    /// Convert a field to Rust type
    fn field_to_rust(&self, field: &FieldInfo) -> String {
        let rust_type = self.python_type_to_rust(&field.type_name);
        let sanitized_name = sanitize_identifier(&field.name);

        if field.is_optional {
            format!("    pub {}: Option<{}>", sanitized_name, rust_type)
        } else {
            format!("    pub {}: {}", sanitized_name, rust_type)
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
        let sanitized_name = sanitize_identifier(&node.name);
        let func_name = format_ident!("{}_node", sanitized_name);
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
        // Build execution order from edges
        let mut visited = std::collections::HashSet::new();
        let mut sequence = Vec::new();

        // Start from entry point
        let mut current = self.graph_info.entry_point.clone();

        loop {
            // Skip special nodes
            if current == "__start__" || current == "__end__" {
                // Find next node from edges
                if let Some(edge) = self.graph_info.edges.iter().find(|e| e.from == current) {
                    current = edge.to.clone();
                    continue;
                } else {
                    break;
                }
            }

            // Avoid infinite loops
            if visited.contains(&current) {
                break;
            }
            visited.insert(current.clone());

            // Add node to sequence
            let sanitized = sanitize_identifier(&current);
            sequence.push(format!("    state = {}_node(state).await?;", sanitized));

            // Find next node
            if let Some(edge) = self.graph_info.edges.iter().find(|e| e.from == current) {
                current = edge.to.clone();
            } else {
                break;
            }
        }

        if sequence.is_empty() {
            // Fallback to all nodes if no edges defined
            self.graph_info
                .nodes
                .iter()
                .map(|node| {
                    let sanitized = sanitize_identifier(&node.name);
                    format!("    state = {}_node(state).await?;", sanitized)
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            sequence.join("\n")
        }
    }
}

/// Sanitize Python identifiers to be valid Rust identifiers
fn sanitize_identifier(name: &str) -> String {
    let mut result = String::new();

    for (i, ch) in name.chars().enumerate() {
        if i == 0 {
            // First character must be alphabetic or underscore
            if ch.is_alphabetic() || ch == '_' {
                result.push(ch);
            } else {
                result.push('_');
                if ch.is_alphanumeric() {
                    result.push(ch);
                }
            }
        } else {
            // Subsequent characters can be alphanumeric or underscore
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
            } else {
                result.push('_');
            }
        }
    }

    // Check for Rust keywords and add underscore suffix if needed
    match result.as_str() {
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" |
        "extern" | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" |
        "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" |
        "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" |
        "type" | "unsafe" | "use" | "where" | "while" | "async" | "await" |
        "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro" |
        "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield" |
        "try" => format!("{}_", result),
        _ => result,
    }
}

/// Generate Rust code from JSON graph info
pub fn generate_from_json(json: &str) -> Result<String> {
    let graph_info: GraphInfo = serde_json::from_str(json)?;
    let generator = CodeGenerator::new(graph_info);
    generator.generate_rust_code()
}