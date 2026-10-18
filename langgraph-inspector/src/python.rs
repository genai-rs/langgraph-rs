use super::*;
use pyo3::prelude::*;

fn value_error(message: impl Into<String>) -> PyErr {
    PyErr::new::<pyo3::exceptions::PyValueError, _>(message.into())
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

    // CompiledStateGraph.nodes contains Pregel runtime nodes, including
    // sentinels. Its builder holds the user-declared StateNodeSpec values.
    let builder_nodes = graph
        .getattr("builder")
        .and_then(|builder| builder.getattr("nodes"));
    let uses_builder_nodes = builder_nodes.is_ok();
    let nodes_dict = match builder_nodes {
        Ok(nodes) => nodes,
        Err(_) => graph.getattr("nodes")?,
    };

    // Iterate over nodes
    let items = nodes_dict.call_method0("items")?;
    for item in items.try_iter()? {
        let item = item?;
        let tuple = item.cast::<pyo3::types::PyTuple>()?;
        let name: String = tuple.get_item(0)?.extract()?;
        if uses_builder_nodes && matches!(name.as_str(), "__start__" | "__end__") {
            return Err(value_error(format!(
                "compiled StateGraph builder contains reserved sentinel node '{name}'"
            )));
        }
        let node_spec = tuple.get_item(1)?;
        let node_obj = node_spec
            .getattr("runnable")
            .unwrap_or_else(|_| node_spec.clone());

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
    let callable = extract_node_callable(node_obj);
    if let Ok(name) = callable.getattr("__name__") {
        return name.extract();
    }

    if let Ok(name) = node_obj.getattr("name") {
        return name.extract();
    }

    // If all else fails, return the type name
    Ok(node_obj.get_type().name()?.to_string())
}

fn extract_node_callable<'py>(node_obj: &Bound<'py, PyAny>) -> Bound<'py, PyAny> {
    for attribute in ["func", "afunc"] {
        if let Ok(callable) = node_obj.getattr(attribute) {
            if !callable.is_none() {
                return callable;
            }
        }
    }
    node_obj.clone()
}

/// Extract function signature
fn extract_function_signature(node_obj: &Bound<'_, PyAny>) -> PyResult<String> {
    let func = extract_node_callable(node_obj);

    // Try to get signature using inspect module
    let py = node_obj.py();
    let inspect = py.import("inspect")?;
    let signature = inspect.call_method1("signature", (func,))?;

    signature.call_method0("__str__")?.extract()
}

/// Extract docstring from a function
fn extract_docstring(node_obj: &Bound<'_, PyAny>) -> Option<String> {
    let func = extract_node_callable(node_obj);
    let doc = func.getattr("__doc__").ok()?;

    if doc.is_none() {
        return None;
    }

    doc.extract().ok()
}

/// Extract source code hint for the function
fn extract_source_hint(node_obj: &Bound<'_, PyAny>) -> Option<String> {
    let py = node_obj.py();
    let func = extract_node_callable(node_obj);

    // Try to get source file and line number
    let inspect = py.import("inspect").ok()?;
    let source_file = inspect.call_method1("getfile", (&func,)).ok()?;
    let source_lines = inspect.call_method1("getsourcelines", (&func,)).ok()?;

    let file: String = source_file.extract().ok()?;
    let lines_tuple = source_lines.cast::<pyo3::types::PyTuple>().ok()?;
    let line_num: usize = lines_tuple.get_item(1).ok()?.extract().ok()?;

    Some(format!("{}:{}", file, line_num))
}

/// Extract edges from the graph
fn extract_edges(graph: &Bound<'_, PyAny>) -> PyResult<Vec<EdgeInfo>> {
    let mut edges = Vec::new();

    // LangGraph 0.6 CompiledStateGraph stores declared topology on builder.edges.
    // Its START edge is represented by entry_point; the END edge keeps the
    // existing GraphInfo __end__ sentinel.
    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(waiting_edges) = builder.getattr("waiting_edges") {
            if waiting_edges.try_iter()?.next().transpose()?.is_some() {
                return Err(value_error(
                    "compiled StateGraph join edges are not supported by GraphInfo",
                ));
            }
        }
        if let Ok(builder_edges) = builder.getattr("edges") {
            for edge in builder_edges.try_iter()? {
                let edge = edge?;
                let tuple = edge.cast::<pyo3::types::PyTuple>()?;
                if tuple.len() != 2 {
                    return Err(value_error("compiled graph edge must have two endpoints"));
                }
                let from: String = tuple.get_item(0)?.extract()?;
                let to: String = tuple.get_item(1)?.extract()?;
                if from != "__start__" {
                    edges.push(EdgeInfo {
                        from,
                        to,
                        condition: None,
                    });
                }
            }
            sort_edges(&mut edges);
            return Ok(edges);
        }
        if builder.getattr("nodes").is_ok() {
            return Err(value_error(
                "compiled StateGraph builder does not expose supported edges metadata",
            ));
        }
    }

    // Try to get edges from the graph
    if let Ok(edges_list) = graph.getattr("edges") {
        for edge in edges_list.try_iter()? {
            let edge = edge?;

            // Edge might be a tuple (from, to) or an object
            if let Ok(tuple) = edge.cast::<pyo3::types::PyTuple>() {
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
            for item in items.try_iter()? {
                let item = item?;
                let tuple = item.cast::<pyo3::types::PyTuple>()?;
                let from: String = tuple.get_item(0)?.extract()?;
                let to_list = tuple.get_item(1)?;

                // Handle different edge formats
                for to in to_list.try_iter()? {
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

    sort_edges(&mut edges);
    Ok(edges)
}

fn sort_edges(edges: &mut [EdgeInfo]) {
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
    });
}

/// Extract state schema from TypedDict
fn extract_state_schema(graph: &Bound<'_, PyAny>) -> PyResult<StateSchema> {
    let mut fields = Vec::new();

    // Prefer the declared user state schema over Pregel channels, which also
    // contain internal runtime fields and erase original annotations.
    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(state_schema) = builder.getattr("state_schema") {
            fields = extract_fields_from_schema(&state_schema)?;
            return Ok(StateSchema { fields });
        }
        if builder.getattr("nodes").is_ok() {
            return Err(value_error(
                "compiled StateGraph builder does not expose supported state_schema metadata",
            ));
        }
    }

    // Try to get the state class/schema
    if let Ok(channels) = graph.getattr("channels") {
        // Iterate over channels to extract field information
        let items = channels.call_method0("items")?;
        for item in items.try_iter()? {
            let item = item?;
            let tuple = item.cast::<pyo3::types::PyTuple>()?;
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
        for item in items.try_iter()? {
            let item = item?;
            let tuple = item.cast::<pyo3::types::PyTuple>()?;
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
    // GenericAlias and typing objects expose __origin__; their string form
    // preserves arguments such as list[str] that __name__ would discard.
    if let Ok(origin) = type_obj.getattr("__origin__") {
        if !origin.is_none() {
            let rendered: String = type_obj.str().ok()?.extract().ok()?;
            return Some(
                rendered
                    .strip_prefix("typing.")
                    .unwrap_or(&rendered)
                    .to_owned(),
            );
        }
    }

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
    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(builder_edges) = builder.getattr("edges") {
            let mut entries = Vec::new();
            for edge in builder_edges.try_iter()? {
                let edge = edge?;
                let tuple = edge.cast::<pyo3::types::PyTuple>()?;
                if tuple.len() != 2 {
                    return Err(value_error("compiled graph edge must have two endpoints"));
                }
                let from: String = tuple.get_item(0)?.extract()?;
                if from == "__start__" {
                    entries.push(tuple.get_item(1)?.extract::<String>()?);
                }
            }
            return match entries.as_slice() {
                [entry] if entry != "__end__" => Ok(entry.clone()),
                [] => Err(value_error(
                    "compiled StateGraph builder has no supported __start__ entry edge",
                )),
                _ => Err(value_error(
                    "compiled StateGraph builder has ambiguous __start__ entry edges",
                )),
            };
        }
        if builder.getattr("nodes").is_ok() {
            return Err(value_error(
                "compiled StateGraph builder does not expose supported edges metadata",
            ));
        }
    }

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

    Err(value_error("graph does not expose a supported entry point"))
}

/// Extract conditional edges from the graph
fn extract_conditional_edges(
    graph: &Bound<'_, PyAny>,
) -> PyResult<HashMap<String, ConditionalEdge>> {
    let mut conditional_edges = HashMap::new();

    // CompiledStateGraph stores explicit route maps in builder.branches as
    // BranchSpec values. Reject inferred routes that GraphInfo cannot express.
    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(branches_by_source) = builder.getattr("branches") {
            for item in branches_by_source.call_method0("items")?.try_iter()? {
                let item = item?;
                let tuple = item.cast::<pyo3::types::PyTuple>()?;
                let source: String = tuple.get_item(0)?.extract()?;
                if matches!(source.as_str(), "__start__" | "__end__") {
                    return Err(value_error(format!(
                        "conditional branches from reserved sentinel '{source}' are not supported"
                    )));
                }
                let source_branches = tuple.get_item(1)?;
                let mut condition_func: Option<String> = None;
                let mut routes = HashMap::new();

                for branch_item in source_branches.call_method0("items")?.try_iter()? {
                    let branch_item = branch_item?;
                    let branch_tuple = branch_item.cast::<pyo3::types::PyTuple>()?;
                    let branch = branch_tuple.get_item(1)?;
                    let path = branch.getattr("path")?;
                    let name: String = path.getattr("name")?.extract()?;
                    if condition_func
                        .as_ref()
                        .is_some_and(|existing| existing != &name)
                    {
                        return Err(value_error(format!(
                            "multiple conditional functions on node '{source}' are not representable"
                        )));
                    }
                    condition_func = Some(name);

                    let ends = branch.getattr("ends")?;
                    if ends.is_none() {
                        return Err(value_error(format!(
                            "conditional branch on node '{source}' has no explicit route map"
                        )));
                    }
                    for route in ends.call_method0("items")?.try_iter()? {
                        let route = route?;
                        let route_tuple = route.cast::<pyo3::types::PyTuple>()?;
                        let label: String = route_tuple.get_item(0)?.extract()?;
                        let target: String = route_tuple.get_item(1)?.extract()?;
                        if routes
                            .insert(label.clone(), target.clone())
                            .is_some_and(|previous| previous != target)
                        {
                            return Err(value_error(format!(
                                "conditional route '{label}' on node '{source}' has conflicting targets"
                            )));
                        }
                    }
                }

                if routes.is_empty() {
                    return Err(value_error(format!(
                        "conditional branches on node '{source}' have no explicit routes"
                    )));
                }
                conditional_edges.insert(
                    source,
                    ConditionalEdge {
                        condition_func: condition_func
                            .unwrap_or_else(|| "unknown_condition".into()),
                        branches: routes,
                    },
                );
            }
            return Ok(conditional_edges);
        }
        if builder.getattr("nodes").is_ok() {
            return Err(value_error(
                "compiled StateGraph builder does not expose supported branches metadata",
            ));
        }
    }

    // Try to get conditional edges from builder
    if let Ok(builder) = graph.getattr("builder") {
        if let Ok(cond_edges_dict) = builder.getattr("_conditional_edges") {
            let items = cond_edges_dict.call_method0("items")?;
            for item in items.try_iter()? {
                let item = item?;
                let tuple = item.cast::<pyo3::types::PyTuple>()?;
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
            for item in items.try_iter()? {
                let item = item?;
                let tuple = item.cast::<pyo3::types::PyTuple>()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyModule;

    #[test]
    fn extracts_graph_metadata_with_current_python_api() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                cr#"
from types import SimpleNamespace

def start_node(state):
    return state

graph = SimpleNamespace(
    nodes={'start': SimpleNamespace(func=start_node)},
    edges=[('start', '__end__')],
    channels={'count': SimpleNamespace(value_type=int)},
    entry_point='start',
)
"#,
                c"fixture.py",
                c"fixture",
            )
            .unwrap();
            let graph = module.getattr("graph").unwrap();
            let info: GraphInfo =
                serde_json::from_str(&extract_graph_info(&graph).unwrap()).unwrap();
            assert_eq!(info.nodes[0].name, "start");
            assert_eq!(info.nodes[0].func_name, "start_node");
            assert_eq!(info.nodes[0].signature, "(state)");
            assert_eq!(info.edges[0].to, "__end__");
            assert_eq!(info.state_schema.fields[0].name, "count");
            assert_eq!(info.state_schema.fields[0].type_name, "int");
            assert_eq!(info.entry_point, "start");
        });
    }

    #[test]
    fn extracts_compiled_state_graph_from_builder_without_runtime_metadata() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                cr#"
from types import SimpleNamespace

class State:
    __annotations__ = {'count': int, 'names': list[str]}

def work(state):
    return state

builder = SimpleNamespace(
    nodes={'work': SimpleNamespace(runnable=SimpleNamespace(func=work))},
    edges={('__start__', 'work'), ('work', '__end__')},
    branches={},
    state_schema=State,
)
graph = SimpleNamespace(
    builder=builder,
    nodes={'__start__': object(), 'work': object()},
    channels={'__pregel_tasks': object()},
    entry_point='__start__',
)
"#,
                c"fixture.py",
                c"fixture",
            )
            .unwrap();
            let graph = module.getattr("graph").unwrap();
            let info: GraphInfo =
                serde_json::from_str(&extract_graph_info(&graph).unwrap()).unwrap();

            assert_eq!(info.nodes.len(), 1);
            assert_eq!(info.nodes[0].name, "work");
            assert_eq!(info.nodes[0].func_name, "work");
            assert_eq!(info.entry_point, "work");
            assert_eq!(info.edges.len(), 1);
            assert_eq!(info.edges[0].from, "work");
            assert_eq!(info.edges[0].to, "__end__");
            assert_eq!(info.state_schema.fields.len(), 2);
            assert_eq!(info.state_schema.fields[0].type_name, "int");
            assert_eq!(info.state_schema.fields[1].type_name, "list[str]");
        });
    }

    #[test]
    fn rejects_compiled_state_graph_without_a_unique_entry_edge() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                cr#"
from types import SimpleNamespace

class State:
    __annotations__ = {'count': int}

def work(state):
    return state

builder = SimpleNamespace(
    nodes={'work': SimpleNamespace(runnable=SimpleNamespace(func=work))},
    edges={('__start__', 'work'), ('__start__', 'other')},
    branches={},
    state_schema=State,
)
graph = SimpleNamespace(builder=builder)
"#,
                c"fixture.py",
                c"fixture",
            )
            .unwrap();
            let graph = module.getattr("graph").unwrap();
            let error = extract_graph_info(&graph).unwrap_err().to_string();
            assert!(error.contains("ambiguous __start__ entry edges"), "{error}");
        });
    }

    #[test]
    fn rejects_conditional_branches_without_explicit_routes() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                cr#"
from types import SimpleNamespace

class State:
    __annotations__ = {'count': int}

def work(state):
    return state

builder = SimpleNamespace(
    nodes={'work': SimpleNamespace(runnable=SimpleNamespace(func=work))},
    edges={('__start__', 'work')},
    branches={
        'work': {
            'route': SimpleNamespace(
                path=SimpleNamespace(name='route'),
                ends=None,
            )
        }
    },
    state_schema=State,
)
graph = SimpleNamespace(builder=builder)
"#,
                c"fixture.py",
                c"fixture",
            )
            .unwrap();
            let graph = module.getattr("graph").unwrap();
            let error = extract_graph_info(&graph).unwrap_err().to_string();
            assert!(error.contains("no explicit route map"), "{error}");
        });
    }

    #[test]
    fn rejects_compiled_graph_without_user_state_schema() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                cr#"
from types import SimpleNamespace

def work(state):
    return state

builder = SimpleNamespace(
    nodes={'work': SimpleNamespace(runnable=SimpleNamespace(func=work))},
    edges={('__start__', 'work')},
    branches={},
)
graph = SimpleNamespace(
    builder=builder,
    channels={'__pregel_tasks': object()},
)
"#,
                c"fixture.py",
                c"fixture",
            )
            .unwrap();
            let graph = module.getattr("graph").unwrap();
            let error = extract_graph_info(&graph).unwrap_err().to_string();
            assert!(error.contains("supported state_schema metadata"), "{error}");
        });
    }
}
