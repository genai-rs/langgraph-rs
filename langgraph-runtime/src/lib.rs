use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// Trait for all graph nodes
#[async_trait]
pub trait GraphNode: Send + Sync {
    async fn execute(&self, state: GraphState) -> Result<GraphState>;
    fn name(&self) -> &str;
}

/// Base state type for graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {
    pub data: HashMap<String, Value>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub current_node: String,
    pub execution_path: Vec<String>,
    pub errors: Vec<String>,
}

impl GraphState {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            metadata: GraphMetadata {
                current_node: String::from("__start__"),
                execution_path: Vec::new(),
                errors: Vec::new(),
            },
        }
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
        self.data
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key))?
            .clone()
            .try_into()
            .map_err(|e| anyhow::anyhow!("Failed to deserialize: {}", e))
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        self.data
            .insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }
}

/// Graph executor
pub struct GraphExecutor {
    nodes: HashMap<String, Arc<dyn GraphNode>>,
    edges: Vec<Edge>,
    entry_point: String,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub condition: Option<Box<dyn Fn(&GraphState) -> bool + Send + Sync>>,
}

impl GraphExecutor {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            entry_point: String::from("__start__"),
        }
    }

    pub fn add_node<N: GraphNode + 'static>(&mut self, node: N) {
        self.nodes.insert(node.name().to_string(), Arc::new(node));
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges.push(Edge {
            from: from.to_string(),
            to: to.to_string(),
            condition: None,
        });
    }

    pub fn set_entry_point(&mut self, node: &str) {
        self.entry_point = node.to_string();
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, mut state: GraphState) -> Result<GraphState> {
        info!("Starting graph execution");

        let mut current = self.entry_point.clone();

        loop {
            // Update metadata
            state.metadata.current_node = current.clone();
            state.metadata.execution_path.push(current.clone());

            // Special handling for terminal nodes
            if current == "__end__" {
                info!("Reached end node");
                break;
            }

            // Execute current node
            if let Some(node) = self.nodes.get(&current) {
                debug!("Executing node: {}", current);
                state = node.execute(state).await?;
            }

            // Find next node
            let next = self.find_next_node(&current, &state);
            match next {
                Some(next_node) => {
                    debug!("Moving to next node: {}", next_node);
                    current = next_node;
                }
                None => {
                    info!("No next node found, ending execution");
                    break;
                }
            }
        }

        Ok(state)
    }

    fn find_next_node(&self, current: &str, state: &GraphState) -> Option<String> {
        for edge in &self.edges {
            if edge.from == current {
                if let Some(condition) = &edge.condition {
                    if condition(state) {
                        return Some(edge.to.clone());
                    }
                } else {
                    return Some(edge.to.clone());
                }
            }
        }
        None
    }
}

/// Tool trait for external integrations
#[async_trait]
pub trait Tool: Send + Sync {
    async fn invoke(&self, input: Value) -> Result<Value>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

/// LLM integration trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
    async fn chat(&self, messages: Vec<Message>) -> Result<Message>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// OpenAI provider implementation
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await?;

        let json: Value = response.json().await?;
        Ok(json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn chat(&self, messages: Vec<Message>) -> Result<Message> {
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages
            }))
            .send()
            .await?;

        let json: Value = response.json().await?;
        Ok(Message {
            role: "assistant".to_string(),
            content: json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        })
    }
}
