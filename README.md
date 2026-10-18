# langgraph-rs

[![CI](https://github.com/genai-rs/langgraph-rs/workflows/CI/badge.svg)](https://github.com/genai-rs/langgraph-rs/actions)
[![codecov](https://codecov.io/gh/genai-rs/langgraph-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/genai-rs/langgraph-rs)
[![Crates.io](https://img.shields.io/crates/v/langgraph-rs)](https://crates.io/crates/langgraph-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Python](https://img.shields.io/badge/python-3.10%2B-blue.svg)](https://www.python.org/)

Experimental tooling for inspecting LangGraph metadata and scaffolding Rust code.

## Overview

`langgraph-rs` is an experimental project. Its current CLI cannot extract graph metadata from a Python workflow, and generated node and routing functions are scaffolds that require implementation. It does not yet convert a workflow's behavior into equivalent Rust code.

The Python package metadata in `pyproject.toml` requires Python 3.10 or newer. The Rust workspace does not declare a minimum supported Rust version in `Cargo.toml`.

### Current capabilities

| Capability | Status |
| --- | --- |
| CLI help and version output | Supported |
| CLI inspection of a Python workflow | Not supported: extraction is not implemented |
| CLI conversion of a Python workflow | Not supported: extraction is not implemented |
| Generated node and conditional-routing logic | Scaffolding only; you must implement the behavior |
| Conversion validation | Not supported: validation is not implemented |
| Performance or production guarantees | Not established |

The Rust generator can produce scaffolding from graph metadata supplied through its Rust API. This does not make the CLI able to inspect Python workflows or provide semantic conversion.

**Release:** v0.1.2 - [Release Notes](https://github.com/genai-rs/langgraph-rs/releases/tag/v0.1.2)

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
# Show available CLI commands and options
langgraph-rs --help

# Check the installed CLI version
langgraph-rs --version

# Show the syntax for the inspection command
langgraph-rs inspect --help
```

The workflow commands are placeholders while Python graph extraction is unimplemented. `convert`, `inspect`, and `visualize` fail for valid Python input with an extraction error; `validate` reports that validation is not implemented. Do not treat generated scaffolding as working node logic or deploy it as an equivalent workflow.

## Example

### Example input (Python LangGraph)

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

This is an input example only. The current CLI cannot extract it, and the generator does not translate the Python node body or produce an equivalent executable workflow. Any generated node logic needs to be implemented and verified by the user.

## Examples

The repository includes Python example scripts showing LangGraph patterns. They are source examples, not verified conversions or production-ready applications:

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

No reproducible end-to-end comparison of equivalent Python and Rust workflows is available. See [Performance](docs/performance.md) for the benchmark status.

## Selected Type Mappings

The generator API contains mappings for common Python type names. They are not evidence that a Python workflow can currently be extracted or semantically converted by the CLI.

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

Deployment examples in the repository are illustrative and do not establish production readiness for this experimental tool:

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
- [Deployment Guide](docs/deployment.md) - Deployment examples; these do not establish production readiness
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

## Release checks

Rust CI builds and tests every workspace crate and runs Clippy with warnings treated as errors. The required `CI` check fails if any build, test, or coverage job fails. Rust dependencies are recorded in `Cargo.lock`; CI and release builds use `--locked`.

To publish a release, merge a passing PR that updates the workspace and Python package version, then push the matching `v<version>` tag. The release workflow runs the full CI suite on the tagged source, builds and smoke-tests the CLI on all four supported platforms, and only then creates or updates the GitHub release. The manual Release workflow can rebuild an existing tag. Tag and package versions must match.

Version `v0.1.0` did not compile and its existing GitHub release has no binaries. Use `v0.1.2` or a later verified release.

The standalone CLI uses the inspector metadata types without linking CPython. Python bindings remain available through the inspector crate's default `python` feature. Release checks reject macOS binaries that depend on non-system libraries or paths from the build machine.
