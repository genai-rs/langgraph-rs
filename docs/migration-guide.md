# Migration Guide: Python LangGraph to Rust

## Current status

`langgraph-rs` is experimental graph-inspection and scaffolding tooling, not a semantic transpiler. The CLI's Python graph extraction is not implemented, so `convert`, `inspect`, and `visualize` currently return an extraction error for a valid Python file. `validate` is also not implemented.

The generator can produce Rust scaffolding when supplied graph metadata through its Rust API. Generated node bodies and conditional routing are placeholders; they do not preserve the Python node behavior. There is no automatic migration path yet.

| Task | Current status |
| --- | --- |
| Inspect a Python workflow from the CLI | Unsupported: extraction is not implemented |
| Convert Python node and routing behavior | Unsupported: semantic conversion is not implemented |
| Generate Rust scaffolding from supplied metadata | Available through the Rust generator API; experimental |
| Validate Python and Rust behavior equivalence | Unsupported: validation is not implemented |
| Deploy generated code as an equivalent workflow | Not supported; node logic must be written and verified by you |

## Install and inspect the CLI syntax

From a checkout of this repository, install the CLI with Cargo:

```bash
cargo install --path langgraph-cli
```

These commands show the actual supported command syntax without attempting an unimplemented workflow operation:

```bash
langgraph-rs --help
langgraph-rs inspect --help
langgraph-rs convert --help
```

The CLI takes a positional Python path for inspection, for example `langgraph-rs inspect workflow.py`. That command currently fails because Python graph extraction is not implemented. The help text describes command arguments, not a promise that the underlying workflow operation is available.

## Manual migration

Until extraction and semantic conversion are implemented, use the LangGraph documentation and migrate workflows manually. Treat any generator output as a starting scaffold only:

1. Recreate the state model in Rust and check that optional fields and values retain the original meaning.
2. Implement each node's behavior in Rust. Generated node functions return an explicit unimplemented error until you replace their editable scaffold body; the Python function body is not translated.
3. Implement conditional routing explicitly. Generated routing functions currently return a placeholder route.
4. Recreate entry, exit, retry, and error behavior, then add tests that exercise the same inputs and observable outputs in both implementations.
5. Review dependencies, secrets handling, persistence, and deployment separately before using a manually migrated workflow.

Do not infer behavioral equivalence, production readiness, or a performance improvement from generated scaffolding. The project does not currently provide an end-to-end equivalence validator or a reproducible Python-versus-Rust benchmark. See [Performance](performance.md) for details.

## Python version

The Python package metadata requires Python 3.10 or newer (`requires-python = ">=3.10"` in `pyproject.toml`). The Rust CLI is built with Cargo; the workspace does not declare a minimum supported Rust version in `Cargo.toml`.
