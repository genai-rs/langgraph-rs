use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod type_mapping;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphInfo {
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<EdgeInfo>,
    pub state_schema: StateSchema,
    pub entry_point: String,
    pub conditional_edges: HashMap<String, ConditionalEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeInfo {
    pub name: String,
    pub func_name: String,
    pub signature: String,
    pub docstring: Option<String>,
    pub source_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeInfo {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateSchema {
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
    pub is_optional: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalEdge {
    pub condition_func: String,
    pub branches: HashMap<String, String>,
}

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::{extract_graph_info, trace_execution};
