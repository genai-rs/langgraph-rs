use langgraph_inspector::GraphInfo;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "langgraph-python-extraction-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn python_executable() -> String {
    std::env::var("LANGGRAPH_PYTHON")
        .expect("set LANGGRAPH_PYTHON to the venv with the local inspector package installed")
}

fn run_inspect(cwd: &Path, source: &Path, graph_object: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_langgraph-rs"))
        .current_dir(cwd)
        .env("LANGGRAPH_PYTHON", python_executable())
        .env_remove("RUST_LOG")
        .args([
            "inspect",
            source.to_str().unwrap(),
            "--graph-object",
            graph_object,
        ])
        .output()
        .unwrap()
}

fn inspect(source: &Path, cwd: &Path) -> GraphInfo {
    let output = run_inspect(cwd, source, "graph");
    assert!(
        output.status.success(),
        "inspect failed for {}\nstdout: {}\nstderr: {}",
        source.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI output must deserialize as GraphInfo")
}

fn node_names(graph: &GraphInfo) -> BTreeSet<&str> {
    graph.nodes.iter().map(|node| node.name.as_str()).collect()
}

fn edge_pairs(graph: &GraphInfo) -> BTreeSet<(&str, &str)> {
    graph
        .edges
        .iter()
        .map(|edge| (edge.from.as_str(), edge.to.as_str()))
        .collect()
}

#[test]
fn installed_helper_extracts_distinct_linear_and_conditional_graphs_outside_repo() {
    let temp = TempDir::new();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
    let synthetic_fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/fixtures/metadata-extraction/graph.py");
    let synthetic = inspect(&synthetic_fixture, &temp.0);
    let linear = inspect(&fixtures.join("linear_workflow.py"), &temp.0);
    let conditional = inspect(&fixtures.join("conditional_workflow.py"), &temp.0);

    assert_eq!(node_names(&synthetic), BTreeSet::from(["start"]));
    assert_eq!(synthetic.entry_point, "start");
    assert_eq!(
        edge_pairs(&synthetic),
        BTreeSet::from([("start", "__end__")])
    );
    assert_eq!(synthetic.nodes[0].func_name, "start_node");
    assert_eq!(synthetic.nodes[0].signature, "(state)");
    assert!(synthetic.nodes[0]
        .source_hint
        .as_deref()
        .is_some_and(|hint| hint.ends_with("graph.py:4")));
    assert!(synthetic
        .state_schema
        .fields
        .iter()
        .any(|field| field.name == "count" && field.type_name == "int"));

    let linear_nodes = node_names(&linear);
    assert!(linear_nodes.contains("a"));
    assert!(linear_nodes.contains("b"));
    assert!(linear_nodes.contains("c"));
    assert!(!linear_nodes.contains("__start__"));
    assert!(linear
        .nodes
        .iter()
        .all(|node| node.func_name != "PregelNode"));
    assert_eq!(linear.entry_point, "a");
    assert_eq!(
        edge_pairs(&linear),
        BTreeSet::from([("a", "b"), ("b", "c"), ("c", "__end__")])
    );
    assert!(linear
        .state_schema
        .fields
        .iter()
        .any(|field| field.name == "counter" && field.type_name == "int"));
    assert!(linear
        .state_schema
        .fields
        .iter()
        .any(|field| field.name == "messages" && field.type_name == "list[str]"));
    assert!(linear
        .state_schema
        .fields
        .iter()
        .all(|field| field.name != "__start__" && field.name != "__pregel_tasks"));

    let conditional_nodes = node_names(&conditional);
    for name in ["start", "high", "low", "end"] {
        assert!(conditional_nodes.contains(name), "missing node {name:?}");
    }
    assert_ne!(linear_nodes, conditional_nodes);
    assert_ne!(edge_pairs(&linear), edge_pairs(&conditional));
    assert!(!conditional_nodes.contains("__start__"));
    assert!(conditional
        .nodes
        .iter()
        .all(|node| node.func_name != "PregelNode"));
    assert_eq!(conditional.entry_point, "start");
    assert!(
        conditional.conditional_edges.contains_key("start"),
        "conditional routes were not extracted: {:?}",
        conditional.conditional_edges
    );
    let start_routes = &conditional.conditional_edges["start"].branches;
    assert!(start_routes.contains_key("high"));
    assert!(start_routes.contains_key("low"));
    assert_eq!(
        conditional.conditional_edges["start"].condition_func,
        "route_based_on_value"
    );
    assert_eq!(
        conditional.conditional_edges["high"].branches,
        std::collections::HashMap::from([
            ("start".to_owned(), "start".to_owned()),
            ("end".to_owned(), "end".to_owned()),
        ])
    );
    assert_eq!(
        conditional.conditional_edges["low"].branches,
        conditional.conditional_edges["high"].branches
    );
    assert!(conditional
        .edges
        .iter()
        .any(|edge| edge.from == "end" && edge.to == "__end__"));
    assert_eq!(
        conditional
            .state_schema
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field.type_name.as_str()))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ("value", "int"),
            ("path_taken", "list[str]"),
            ("should_continue", "bool"),
        ])
    );
}

#[test]
fn helper_reports_missing_selected_object_without_graph_output() {
    let temp = TempDir::new();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/linear_workflow.py");
    let output = run_inspect(&temp.0, &fixture, "not_a_graph");
    assert!(!output.status.success(), "unexpected success: {output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("graph object 'not_a_graph' was not found"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "unexpected stdout: {output:?}");
}

#[test]
fn helper_isolates_python_and_native_stdout_writes() {
    let temp = TempDir::new();
    let fixture = temp.0.join("stdout_workflow.py");
    fs::write(
        &fixture,
        r#"
import os
from types import SimpleNamespace

def start_node(state):
    return state

print("python stdout must not corrupt the protocol")
os.write(1, b"native stdout must not corrupt the protocol\\n")
graph = SimpleNamespace(
    nodes={"start": SimpleNamespace(func=start_node)},
    edges=[("start", "__end__")],
    channels={"count": SimpleNamespace(value_type=int)},
    entry_point="start",
)
"#,
    )
    .unwrap();

    let graph = inspect(&fixture, &temp.0);
    assert_eq!(node_names(&graph), BTreeSet::from(["start"]));
    assert_eq!(graph.entry_point, "start");
}

#[test]
fn extracts_async_node_callable_and_signature() {
    let temp = TempDir::new();
    let fixture = temp.0.join("async_workflow.py");
    fs::write(
        &fixture,
        r#"
from typing import TypedDict
from langgraph.graph import END, START, StateGraph

class State(TypedDict):
    count: int

async def afunc(state: State) -> State:
    """Asynchronous workflow node."""
    return state

workflow = StateGraph(State)
workflow.add_node("async_node", afunc)
workflow.add_edge(START, "async_node")
workflow.add_edge("async_node", END)
graph = workflow.compile()
"#,
    )
    .unwrap();

    let graph = inspect(&fixture, &temp.0);
    let node = graph
        .nodes
        .iter()
        .find(|node| node.name == "async_node")
        .unwrap();
    assert_eq!(node.func_name, "afunc");
    assert!(node.signature.contains("state:"), "{}", node.signature);
    assert_eq!(
        node.docstring.as_deref(),
        Some("Asynchronous workflow node.")
    );
    assert!(node
        .source_hint
        .as_deref()
        .is_some_and(|hint| hint.ends_with("async_workflow.py:8")));
}

#[test]
fn rejects_compiled_graph_with_join_edges() {
    let temp = TempDir::new();
    let fixture = temp.0.join("join_workflow.py");
    fs::write(
        &fixture,
        r#"
from typing import TypedDict
from langgraph.graph import END, START, StateGraph

class State(TypedDict):
    count: int

def node(state: State) -> State:
    return state

workflow = StateGraph(State)
for name in ("a", "b", "c"):
    workflow.add_node(name, node)
workflow.add_edge(START, "a")
workflow.add_edge(START, "b")
workflow.add_edge(["a", "b"], "c")
workflow.add_edge("c", END)
graph = workflow.compile()
"#,
    )
    .unwrap();

    let output = run_inspect(&temp.0, &fixture, "graph");
    assert!(!output.status.success(), "unexpected success: {output:?}");
    assert!(output.stdout.is_empty(), "unexpected stdout: {output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("join edges are not supported by GraphInfo"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rejects_conditional_edges_from_start_sentinel() {
    let temp = TempDir::new();
    let fixture = temp.0.join("conditional_start_workflow.py");
    fs::write(
        &fixture,
        r#"
from typing import Literal, TypedDict
from langgraph.graph import END, START, StateGraph

class State(TypedDict):
    count: int

def node(state: State) -> State:
    return state

def route(state: State) -> Literal["a"]:
    return "a"

workflow = StateGraph(State)
workflow.add_node("a", node)
workflow.add_edge(START, "a")
workflow.add_edge("a", END)
workflow.add_conditional_edges(START, route, {"a": "a"})
graph = workflow.compile()
"#,
    )
    .unwrap();

    let output = run_inspect(&temp.0, &fixture, "graph");
    assert!(!output.status.success(), "unexpected success: {output:?}");
    assert!(output.stdout.is_empty(), "unexpected stdout: {output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("conditional branches from reserved sentinel '__start__' are not supported"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn conversion_still_generates_scaffold_node_bodies() {
    let temp = TempDir::new();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/linear_workflow.py");
    let output_dir = temp.0.join("generated");
    let output = Command::new(env!("CARGO_BIN_EXE_langgraph-rs"))
        .current_dir(&temp.0)
        .env("LANGGRAPH_PYTHON", python_executable())
        .env_remove("RUST_LOG")
        .args([
            "convert",
            fixture.to_str().unwrap(),
            "--graph-object",
            "graph",
            "--output",
            output_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "convert failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = fs::read_to_string(output_dir.join("src/lib.rs")).unwrap();
    assert!(generated.contains("Implement this node's business logic here"));
    assert!(generated.contains("Node 'a' is not implemented"));
}
