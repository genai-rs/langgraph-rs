use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod type_mapping;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInfo {
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<EdgeInfo>,
    pub state_schema: StateSchema,
    pub entry_point: String,
    pub conditional_edges: HashMap<String, ConditionalEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub func_name: String,
    pub signature: String,
    pub docstring: Option<String>,
    pub source_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInfo {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSchema {
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
    pub is_optional: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalEdge {
    pub condition_func: String,
    pub branches: HashMap<String, String>,
}

/// Extract metadata from a LangGraph instance
#[pyfunction]
pub fn extract_graph_info(graph: &Bound<'_, PyAny>) -> PyResult<String> {
    // Note: graph is already compiled in LangGraph, so we don't call compile() again
    let nodes = extract_nodes(graph)?;
    let edges = extract_edges(graph)?;
    let state_schema = extract_state_schema(graph)?;
    let entry_point = extract_entry_point(graph)?;
    let conditional_edges = extract_conditional_edges(graph)?;

    let graph_info = GraphInfo {
        nodes,
        edges,
        state_schema,
        entry_point,
        conditional_edges,
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&graph_info)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(json)
}

/// Extract node information from the graph
fn extract_nodes(graph: &Bound<'_, PyAny>) -> PyResult<Vec<NodeInfo>> {
    let mut nodes = Vec::new();

    // Try to get nodes from the graph
    // LangGraph compiled graphs have a nodes attribute
    let nodes_dict = graph.getattr("nodes").or_else(|_| {
        // If that fails, try getting it from the builder/workflow
        graph.getattr("builder")?.getattr("nodes")
    })?;

    // Iterate over nodes
    let items = nodes_dict.call_method0("items")?;
    for item in items.iter()? {
        let item = item?;
        let tuple = item.downcast::<pyo3::types::PyTuple>()?;
        let name: String = tuple.get_item(0)?.extract()?;
        let node_obj = tuple.get_item(1)?;

        let func_name = extract_function_name(&node_obj)?;
        let signature = extract_function_signature(&node_obj).unwrap_or_default();
        let docstring = extract_docstring(&node_obj);
        let source_hint = extract_source_hint(&node_obj);

        nodes.push(NodeInfo {
            name,
            func_name,
            signature,
            docstring,
            source_hint,
        });
    }

    Ok(nodes)
}

/// Extract function name from a node object
fn extract_function_name(node_obj: &Bound<'_, PyAny>) -> PyResult<String> {
    // Try different ways to get the function name
    if let Ok(func) = node_obj.getattr("func") {
        if let Ok(name) = func.getattr("__name__") {
            return name.extract();
        }
    }

    if let Ok(name) = node_obj.getattr("__name__") {
        return name.extract();
    }

    // If all else fails, return the type name
    Ok(node_obj.get_type().name()?.to_string())
}

/// Extract function signature
fn extract_function_signature(node_obj: &Bound<'_, PyAny>) -> PyResult<String> {
    let func = node_obj
        .getattr("func")
        .unwrap_or_else(|_| node_obj.clone());

    // Try to get signature using inspect module
    let py = node_obj.py();
    let inspect = py.import_bound("inspect")?;
    let signature = inspect.call_method1("signature", (func,))?;

    signature.call_method0("__str__")?.extract()
}

/// Extract docstring from a function
fn extract_docstring(node_obj: &Bound<'_, PyAny>) -> Option<String> {
    let func = node_obj
        .getattr("func")
        .ok()
        .unwrap_or_else(|| node_obj.clone());
    let doc = func.getattr("__doc__").ok()?;

    if doc.is_none() {
        return None;
    }

    doc.extract().ok()
}

/// Extract source code hint for the function
fn extract_source_hint(node_obj: &Bound<'_, PyAny>) -> Option<String> {
    let py = node_obj.py();
    let func = node_obj
        .getattr("func")
        .ok()
        .unwrap_or_else(|| node_obj.clone());

    // Try to get source file and line number
    let inspect = py.import_bound("inspect").ok()?;
    let source_file = inspect.call_method1("getfile", (&func,)).ok()?;
    let source_lines = inspect.call_method1("getsourcelines", (&func,)).ok()?;

    let file: String = source_file.extract().ok()?;
    let lines_tuple = source_lines.downcast::<pyo3::types::PyTuple>().ok()?;
    let line_num: usize = lines_tuple.get_item(1).ok()?.extract().ok()?;

    Some(format!("{}:{}", file, line_num))
}

/// Extract edges from the graph
fn extract_edges(graph: &Bound<'_, PyAny>) -> PyResult<Vec<EdgeInfo>> {
    let mut edges = Vec::new();

    // Try to get edges from the graph
    if let Ok(edges_list) = graph.getattr("edges") {
        for edge in edges_list.iter()? {
            let edge = edge?;

            // Edge might be a tuple (from, to) or an object
            if let Ok(tuple) = edge.downcast::<pyo3::types::PyTuple>() {
                let from: String = tuple.get_item(0)?.extract()?;
                let to: String = tuple.get_item(1)?.extract()?;

                edges.push(EdgeInfo {
                    from,
                    to,
                    condition: None,
                });
            }
        }
    }

    // Try to extract from _edges or other internal attributes
    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(edges_dict) = builder.getattr("_edges") {
            let items = edges_dict.call_method0("items")?;
            for item in items.iter()? {
                let item = item?;
                let tuple = item.downcast::<pyo3::types::PyTuple>()?;
                let from: String = tuple.get_item(0)?.extract()?;
                let to_list = tuple.get_item(1)?;

                // Handle different edge formats
                for to in to_list.iter()? {
                    let to_str: String = to?.extract().unwrap_or_default();
                    if !to_str.is_empty() {
                        edges.push(EdgeInfo {
                            from: from.clone(),
                            to: to_str,
                            condition: None,
                        });
                    }
                }
            }
        }
    }

    Ok(edges)
}

/// Extract state schema from TypedDict
fn extract_state_schema(graph: &Bound<'_, PyAny>) -> PyResult<StateSchema> {
    let mut fields = Vec::new();

    // Try to get the state class/schema
    if let Ok(channels) = graph.getattr("channels") {
        // Iterate over channels to extract field information
        let items = channels.call_method0("items")?;
        for item in items.iter()? {
            let item = item?;
            let tuple = item.downcast::<pyo3::types::PyTuple>()?;
            let field_name: String = tuple.get_item(0)?.extract()?;
            let channel_obj = tuple.get_item(1)?;

            // Try to extract type information
            let type_name = extract_type_name(&channel_obj).unwrap_or_else(|| "Any".to_string());

            fields.push(FieldInfo {
                name: field_name,
                type_name,
                is_optional: false, // TODO: Detect optional fields
                default_value: None,
            });
        }
    }

    // Alternative: try to get from builder
    if fields.is_empty() {
        if let Ok(builder) = graph.getattr("builder") {
            if let Ok(state_schema) = builder.getattr("schema") {
                fields = extract_fields_from_schema(&state_schema)?;
            }
        }
    }

    Ok(StateSchema { fields })
}

/// Extract field information from a schema object
fn extract_fields_from_schema(schema: &Bound<'_, PyAny>) -> PyResult<Vec<FieldInfo>> {
    let mut fields = Vec::new();

    // Try to get __annotations__ for TypedDict
    if let Ok(annotations) = schema.getattr("__annotations__") {
        let items = annotations.call_method0("items")?;
        for item in items.iter()? {
            let item = item?;
            let tuple = item.downcast::<pyo3::types::PyTuple>()?;
            let field_name: String = tuple.get_item(0)?.extract()?;
            let type_obj = tuple.get_item(1)?;

            let type_name = get_type_string(&type_obj).unwrap_or_else(|| "Any".to_string());
            let is_optional = type_name.starts_with("Optional[") || type_name.contains("None");

            fields.push(FieldInfo {
                name: field_name,
                type_name,
                is_optional,
                default_value: None,
            });
        }
    }

    Ok(fields)
}

/// Extract type name from a channel or field object
fn extract_type_name(obj: &Bound<'_, PyAny>) -> Option<String> {
    // Try to get the value_type attribute
    if let Ok(value_type) = obj.getattr("value_type") {
        return get_type_string(&value_type);
    }

    // Try to get __class__.__name__
    if let Ok(class) = obj.getattr("__class__") {
        if let Ok(name) = class.getattr("__name__") {
            return name.extract().ok();
        }
    }

    None
}

/// Convert Python type object to string representation
fn get_type_string(type_obj: &Bound<'_, PyAny>) -> Option<String> {
    // Try __name__ first
    if let Ok(name) = type_obj.getattr("__name__") {
        return name.extract().ok();
    }

    // Try __str__
    if let Ok(str_repr) = type_obj.call_method0("__str__") {
        return str_repr.extract().ok();
    }

    // Try __repr__
    if let Ok(repr) = type_obj.call_method0("__repr__") {
        return repr.extract().ok();
    }

    None
}

/// Extract entry point from the graph
fn extract_entry_point(graph: &Bound<'_, PyAny>) -> PyResult<String> {
    // Try multiple ways to get the entry point
    if let Ok(entry) = graph.getattr("entry_point") {
        if let Ok(entry_str) = entry.extract::<String>() {
            return Ok(entry_str);
        }
    }

    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(entry) = builder.getattr("entry_point") {
            return entry.extract();
        }
    }

    // Default entry point
    Ok("__start__".to_string())
}

/// Extract conditional edges from the graph
fn extract_conditional_edges(
    graph: &Bound<'_, PyAny>,
) -> PyResult<HashMap<String, ConditionalEdge>> {
    let mut conditional_edges = HashMap::new();

    // Try to get conditional edges from builder
    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(cond_edges_dict) = builder.getattr("_conditional_edges") {
            let items = cond_edges_dict.call_method0("items")?;
            for item in items.iter()? {
                let item = item?;
                let tuple = item.downcast::<pyo3::types::PyTuple>()?;
                let source_node: String = tuple.get_item(0)?.extract()?;
                let cond_edge_obj = tuple.get_item(1)?;

                // Extract condition function name
                let condition_func = extract_condition_function_name(&cond_edge_obj)?;

                // Extract branch mappings
                let branches = extract_branch_mappings(&cond_edge_obj)?;

                conditional_edges.insert(
                    source_node,
                    ConditionalEdge {
                        condition_func,
                        branches,
                    },
                );
            }
        }
    }

    Ok(conditional_edges)
}

/// Extract condition function name from conditional edge object
fn extract_condition_function_name(cond_edge: &Bound<'_, PyAny>) -> PyResult<String> {
    // Try to get the condition/router function
    if let Ok(func) = cond_edge.getattr("condition") {
        if let Ok(name) = func.getattr("__name__") {
            return name.extract();
        }
    }

    if let Ok(func) = cond_edge.getattr("path_map") {
        if let Ok(name) = func.getattr("__name__") {
            return name.extract();
        }
    }

    Ok("unknown_condition".to_string())
}

/// Extract branch mappings from conditional edge
fn extract_branch_mappings(cond_edge: &Bound<'_, PyAny>) -> PyResult<HashMap<String, String>> {
    let mut branches = HashMap::new();

    // Try to get the path_map or branches
    if let Ok(path_map) = cond_edge.getattr("path_map") {
        if let Ok(items) = path_map.call_method0("items") {
            for item in items.iter()? {
                let item = item?;
                let tuple = item.downcast::<pyo3::types::PyTuple>()?;
                let key: String = tuple.get_item(0)?.extract()?;
                let value: String = tuple.get_item(1)?.extract()?;
                branches.insert(key, value);
            }
        }
    }

    Ok(branches)
}

/// Capture execution trace of a LangGraph
#[pyfunction]
pub fn trace_execution(
    graph: &Bound<'_, PyAny>,
    input_data: &Bound<'_, PyAny>,
) -> PyResult<String> {
    // Compile the graph
    let compiled = graph.call_method0("compile")?;

    // TODO: Implement execution tracing
    // This would involve monkey-patching or wrapping the graph execution

    let result = compiled.call_method1("invoke", (input_data,))?;

    Ok(format!("Execution traced: {:?}", result))
}

/// Python module definition
#[pymodule]
fn langgraph_inspector(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(extract_graph_info, m)?)?;
    m.add_function(wrap_pyfunction!(trace_execution, m)?)?;
    Ok(())
}
