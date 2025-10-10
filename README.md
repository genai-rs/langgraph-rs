# langgraph-rs

[![CI](https://github.com/genai-rs/langgraph-rs/workflows/CI/badge.svg)](https://github.com/genai-rs/langgraph-rs/actions)
[![codecov](https://codecov.io/gh/genai-rs/langgraph-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/genai-rs/langgraph-rs)
[![Crates.io](https://img.shields.io/crates/v/langgraph-rs)](https://crates.io/crates/langgraph-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)](https://www.python.org/)

Convert LangGraph (Python) workflows into high-performance Rust code for production-ready LLM applications.

## Overview

`langgraph-rs` is a transpiler that converts LangGraph Python workflows into efficient Rust implementations, delivering 7-8x performance improvements over Python execution.

**Key Features:**
- 7-8x faster than Python execution
- Memory safety without garbage collection
- True async concurrency with Tokio
- Smaller deployment footprint and lower resource usage
- Type-safe code generation with proper error handling
- Production-ready with Docker and Kubernetes support

**Current Release:** v0.1.0 - [Release Notes](https://github.com/genai-rs/langgraph-rs/releases/tag/v0.1.0)

## Architecture

The workspace is organized into the following crates:

- `langgraph-inspector/` - Python introspection via PyO3
- `langgraph-generator/` - Rust code generation
- `langgraph-runtime/` - Runtime support for generated code
- `langgraph-cli/` - Command-line interface

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/genai-rs/langgraph-rs
cd langgraph-rs

# Build the project
cargo build --release

# Install the CLI
cargo install --path langgraph-cli
```

### Basic Usage

```bash
# Convert a LangGraph workflow to Rust
langgraph-rs convert my_workflow.py --output ./rust_graph/

# Inspect a LangGraph instance
langgraph-rs inspect --graph my_graph.py

# Validate conversion
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

## Examples

The project includes 11 production-ready examples demonstrating various patterns:

- Customer Support Agent (multi-tier routing)
- Content Moderation (filtering pipeline)
- Document Processing (validation and retry)
- Chatbot with Fallback (graceful degradation)
- API Router (circuit breaker and retry)
- Data Validation & Enrichment (quality scoring)
- Order Fulfillment (e-commerce workflow)
- Recommendation Engine (multiple strategies)
- ETL Pipeline (error handling)
- A/B Testing Router (experiment routing)
- Email Campaign (marketing ***REMOVED***)

See [examples/README.md](examples/README.md) for detailed documentation.

## Performance

Benchmarks show 7-8x performance improvement over Python:

| Workflow Pattern | Python (ms) | Rust (ms) | Speedup |
|-----------------|-------------|-----------|---------|
| Linear workflow | 42 | 6 | 7.0x |
| Conditional branching | 45 | 6 | 7.5x |
| Loop with counter | 48 | 6 | 8.0x |
| Average | 45 | 6 | 7.5x |

*Benchmarks run on Apple M1, single-threaded. Excludes LLM API latency.*

## Type Mappings

Comprehensive Python to Rust type conversion:

| Python | Rust |
|--------|------|
| `str` | `String` |
| `int` | `i64` |
| `float` | `f64` |
| `bool` | `bool` |
| `list[T]` | `Vec<T>` |
| `dict[K, V]` | `HashMap<K, V>` |
| `T \| None` | `Option<T>` |
| `Any` | `serde_json::Value` |

## Deployment

Production-ready deployment options:

### Docker

```bash
docker build -t langgraph-rs:latest .
docker run -p 8080:8080 langgraph-rs:latest
```

### Kubernetes

```bash
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
```

### Docker Compose (with monitoring)

```bash
docker-compose up -d
# Grafana available at http://localhost:3000
# Prometheus at http://localhost:9090
```

See [docs/deployment.md](docs/deployment.md) for detailed deployment instructions.

## Documentation

- [Migration Guide](docs/migration-guide.md) - Step-by-step migration from Python
- [Deployment Guide](docs/deployment.md) - Production deployment options
- [Performance Tuning](docs/performance.md) - Optimization techniques
- [Contributing](CONTRIBUTING.md) - Development guidelines
- [Changelog](CHANGELOG.md) - Version history

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

## Contact

- GitHub: [@genai-rs](https://github.com/genai-rs)
- Issues: [GitHub Issues](https://github.com/genai-rs/langgraph-rs/issues)
