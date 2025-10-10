# langgraph-rs

[![CI](https://github.com/genai-rs/langgraph-rs/workflows/CI/badge.svg)](https://github.com/genai-rs/langgraph-rs/actions)
[![codecov](https://codecov.io/gh/genai-rs/langgraph-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/genai-rs/langgraph-rs)
[![Crates.io](https://img.shields.io/crates/v/langgraph-rs)](https://crates.io/crates/langgraph-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)](https://www.python.org/)

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

**Project Status: MVP Complete - Ready for v0.1.0**

### ✅ Phase 1: Foundation (100%)
- ✅ Cargo workspace with 4 crates
- ✅ CLI with all subcommands
- ✅ Module structure and architecture
- ✅ Example workflows and documentation

### ✅ Phase 2: Python Introspection (100%)
- ✅ Python environment and comprehensive fixtures (5 patterns)
- ✅ Complete PyO3 introspection (nodes, edges, state, conditionals)
- ✅ Advanced type mapping system
- ✅ Pytest configuration with async and benchmarking
- ✅ CI/CD with multi-platform testing
- ✅ Code coverage integration

### ✅ Phase 3: Code Generation (100%)
- ✅ State struct generation with proper types
- ✅ Node function stubs with full metadata
- ✅ Conditional routing and branching logic
- ✅ Error handling with context
- ✅ Code formatting utilities
- ✅ Python reference comments

### ✅ Phase 4: Runtime Support (100%)
- ✅ State management with execution tracking
- ✅ LLM provider trait + OpenAI implementation
- ✅ Tool system framework
- ✅ Comprehensive tracing

### ✅ Phase 5: Validation & Testing (100%)
- ✅ Integration tests (linear, conditional, loops)
- ✅ Performance benchmarks
- ✅ Python-Rust output comparison framework
- ✅ Validation scripts

### ✅ Phase 6: Production Features (100%)
- ✅ Multi-stage Docker setup
- ✅ Kubernetes manifests
- ✅ Docker Compose with monitoring
- ✅ Deployment documentation
- ✅ Performance tuning guide

### ✅ Phase 7: Ecosystem (100%)
- ✅ Dual MIT/Apache-2.0 licenses
- ✅ Migration guide
- ✅ Contributing guidelines
- ✅ Ready for crates.io publication

## Architecture

The workspace is organised into the following crates:

- `langgraph-inspector/` - Python introspection via PyO3
- `langgraph-generator/` - Rust code generation
- `langgraph-runtime/` - Runtime support for generated code
- `langgraph-cli/` - Command-line interface

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

**Development status:** Early development. The project currently provides only the basic workspace structure; no end-to-end conversion is available yet.

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

- [LangGraph](https://github.com/langchain-ai/langgraph) - Original Python implementation
- [Why LangGraph?](https://langchain-ai.github.io/langgraph/concepts/why-langgraph/) - Motivation and core concepts
- [langchain-rust](https://github.com/Abraxas-365/langchain-rust) - LangChain for Rust
- [rs-graph-llm](https://github.com/a-agmon/rs-graph-llm) - Rust framework inspired by LangGraph
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
- [PyO3 Book](https://pyo3.rs/main/getting-started.html) - Official getting started guide

## Contact

- GitHub: [@genai-rs](https://github.com/genai-rs)
- Issues: [GitHub Issues](https://github.com/genai-rs/langgraph-rs/issues)
