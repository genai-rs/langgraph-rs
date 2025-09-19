# langgraph-rs 🦀

Convert LangGraph (Python) workflows into high-performance Rust code for production-ready LLM applications.

## Overview

`langgraph-rs` is a transpiler that converts LangGraph Python workflows into efficient Rust implementations.

**Project Goals:**
- 5-10x performance improvement over Python execution
- Memory safety without garbage collection
- True async concurrency for handling multiple requests
- Smaller deployment footprint and lower resource usage
- Type-safe code generation with proper error handling

## Current Implementation Status

✅ **Completed:**
- Basic project structure and Cargo workspace
- CLI skeleton with subcommands (convert, inspect, validate, visualize)
- Module structure for inspector, generator, runtime, and CLI
- Basic type definitions and traits
- Example workflows (Python input and expected Rust output)

🚧 **In Progress:**
- PyO3 introspection implementation
- Actual Python-to-Rust conversion logic
- Runtime execution framework

❌ **Not Yet Implemented:**
- Actual Python workflow parsing
- Code generation from real LangGraph instances
- Validation and testing framework
- LLM provider integrations
- Production optimizations

## Architecture

```
langgraph-rs/
├── langgraph-inspector/    # Python introspection via PyO3
├── langgraph-generator/    # Rust code generation
├── langgraph-runtime/      # Runtime support for generated code
└── langgraph-cli/         # Command-line interface
```

## Quick Start

### Building from Source

```bash
# Clone the repository
git clone https://github.com/genai-rs/langgraph-rs
cd langgraph-rs

# Build the project
cargo build

# Run the CLI
cargo run --bin langgraph-rs -- --help
```

### Basic Usage (Planned)

The following commands are defined but not yet fully functional:

```bash
# Convert a LangGraph workflow to Rust (not yet implemented)
langgraph-rs convert my_workflow.py --output ./rust_graph/

# Inspect a LangGraph instance (returns mock data)
langgraph-rs inspect --graph my_graph.py

# Validate conversion (not yet implemented)
langgraph-rs validate --python ./original.py --rust ./generated/
```

## Example

### Input (Python LangGraph)

```python
from langgraph.graph import StateGraph
from typing import TypedDict

class AgentState(TypedDict):
    messages: list[str]
    context: dict

def llm_node(state: AgentState) -> AgentState:
    # LLM logic here
    return state

graph = StateGraph(AgentState)
graph.add_node("llm", llm_node)
graph.set_entry_point("llm")
graph.set_finish_point("llm")

compiled = graph.compile()
```

### Output (Generated Rust)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentState {
    messages: Vec<String>,
    context: HashMap<String, serde_json::Value>,
}

async fn llm_node(state: AgentState) -> Result<AgentState, anyhow::Error> {
    // LLM logic here
    Ok(state)
}

async fn execute_graph(mut state: AgentState) -> Result<AgentState, anyhow::Error> {
    state = llm_node(state).await?;
    Ok(state)
}
```

## Development Status

🚧 **Early Development** - This project has just been initialized with basic structure. No functional conversion is working yet.

### Next Steps

See [TODO.md](TODO.md) for the detailed development plan. The immediate priorities are:
1. Setting up Python test environment
2. Implementing PyO3 introspection to extract LangGraph metadata
3. Creating the type mapping system
4. Building the code generator

## Why Rust for LLM Services?

- **Performance**: Zero-cost abstractions and no garbage collection overhead
- **Concurrency**: Tokio async runtime handles thousands of concurrent requests
- **Safety**: Memory safety guarantees prevent common production issues
- **Efficiency**: Smaller binaries and lower memory usage reduce costs

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is dual-licensed under MIT OR Apache-2.0.

## Related Projects & Resources

- [LangGraph](https://github.com/langchain-ai/langgraph) – Original Python implementation
- [Why LangGraph?](https://langchain-ai.github.io/langgraph/concepts/why-langgraph/) – Motivation and core concepts
- [langchain-rust](https://github.com/Abraxas-365/langchain-rust) – LangChain for Rust
- [rs-graph-llm](https://github.com/a-agmon/rs-graph-llm) – Rust framework inspired by LangGraph
- [PyO3](https://github.com/PyO3/pyo3) – Rust bindings for Python
- [PyO3 Book](https://pyo3.rs/main/getting-started.html) – Official getting started guide

## Contact

- GitHub: [@genai-rs](https://github.com/genai-rs)
- Issues: [GitHub Issues](https://github.com/genai-rs/langgraph-rs/issues)
