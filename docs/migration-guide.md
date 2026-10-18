# Migration Guide: Python LangGraph to Rust

## Current status

`langgraph-rs` is experimental graph-inspection and scaffolding tooling, not a semantic transpiler. `inspect`, `visualize`, and the extraction step used by `convert` can read an explicitly selected graph object when the chosen Python environment contains the local inspector package and the workflow's dependencies. `validate` is not implemented.

The generator can produce Rust scaffolding when supplied graph metadata through its Rust API. Generated node bodies and conditional routing are placeholders; they do not preserve the Python node behavior. There is no automatic migration path yet.

| Task | Current status |
| --- | --- |
| Inspect graph metadata from a Python workflow | Supported with the local inspector package installed in the selected Python environment |
| Convert Python node and routing behavior | Not supported: conversion generates scaffolding and does not translate behavior |
| Visualize extracted graph metadata | Mermaid and DOT output for declared nodes and ordinary edges |
| Generate Rust scaffolding from supplied metadata | Available through the Rust generator API; experimental |
| Validate Python and Rust behavior equivalence | Unsupported: validation is not implemented |
| Deploy generated code as an equivalent workflow | Not supported; node logic must be written and verified by you |

## Install and inspect the CLI syntax

From a checkout of this repository, install the CLI with Cargo:

```bash
cargo install --path langgraph-cli
```

These commands show the supported command syntax:

```bash
langgraph-rs --help
langgraph-rs inspect --help
langgraph-rs convert --help
```

Install the Python dependencies and build the local inspector package from the repository root:

```bash
uv sync --locked --all-extras
uv pip install --no-deps ./langgraph-inspector
```

Then select the Python interpreter that contains both packages and name the graph explicitly:

```bash
LANGGRAPH_PYTHON=.venv/bin/python langgraph-rs inspect workflow.py --graph-object graph
LANGGRAPH_PYTHON=.venv/bin/python langgraph-rs convert workflow.py --graph-object graph
```

Use `--python PATH` instead of `LANGGRAPH_PYTHON` when selecting an interpreter for one command. The selector accepts dotted attribute lookup, such as `workflow.graph`; it does not evaluate Python expressions. The CLI does not install dependencies automatically. On Windows, the uv environment's interpreter is `.venv\\Scripts\\python.exe`.

## Manual migration

For behavior migration, use the LangGraph documentation and implement workflow behavior manually. Treat any generator output as a starting scaffold only:

1. Recreate the state model in Rust and check that optional fields and values retain the original meaning.
2. Implement each node's behavior in Rust. Generated node functions return an explicit unimplemented error until you replace their editable scaffold body; the Python function body is not translated.
3. Implement conditional routing explicitly. Generated routing functions currently return a placeholder route.
4. Recreate entry, exit, retry, and error behavior, then add tests that exercise the same inputs and observable outputs in both implementations.
5. Review dependencies, secrets handling, persistence, and deployment separately before using a manually migrated workflow.

Do not infer behavioral equivalence, production readiness, or a performance improvement from generated scaffolding. The project does not currently provide an end-to-end equivalence validator or a reproducible Python-versus-Rust benchmark. See [Performance](performance.md) for details.

## Python version

The Python package metadata requires Python 3.10 or newer (`requires-python = ">=3.10"` in `pyproject.toml`). The Rust CLI is built with Cargo; the workspace does not declare a minimum supported Rust version in `Cargo.toml`.
