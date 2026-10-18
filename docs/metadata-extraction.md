# Python metadata extraction protocol (proposal)

**Status:** proposal for supervisor review. This document specifies an interface; it does not enable Python execution in the CLI.

## Decision requested

Approve a subprocess adapter for `inspect` and the extraction step used by `convert`. The adapter keeps the Rust CLI independent of CPython at link and startup time. It uses the selected Python interpreter to load the user's file, then calls the existing `langgraph_inspector.extract_graph_info` PyO3 function. It must not reproduce that function's graph traversal in Python or translate arbitrary Python code into Rust.

The remaining packaging decision is how the `langgraph_inspector` extension and a small `langgraph_inspector_helper` module are installed for the selected interpreter. The protocol below assumes both import from that interpreter's environment. This should be settled before implementing the adapter; it does not require changing the standalone CLI's link dependencies.

## Invocation and interpreter selection

An extraction command supplies a Python file and a required graph object selector:

```text
langgraph-rs inspect examples/workflow.py --graph-object graph
langgraph-rs inspect examples/workflow.py --graph-object workflow.graph \
  --python /path/to/venv/bin/python --timeout 45s
```

`--graph-object` is a dotted sequence of Python identifiers resolved by attribute lookup. It does not accept expressions, calls, indexing, or an inferred default. The command fails if the selector is absent, malformed, or resolves to no object. The helper does not call `compile()` on the selected object; it passes the object to the existing extractor as-is.

Choose the interpreter in this order:

1. An explicit `--python` executable path.
2. `LANGGRAPH_PYTHON`, if set to an executable path.
3. `python3` found on `PATH` (with `python` as the Windows fallback).

The CLI starts the helper only when an extraction operation runs. It does not probe Python while parsing arguments or handling `--help` and `--version`.

The CLI invokes `python -m langgraph_inspector_helper` and sends one UTF-8 JSON request on stdin. The helper imports the requested source file and selected object, then calls `langgraph_inspector.extract_graph_info`. The helper and extension must be importable by the chosen interpreter, and LangGraph dependencies used by the file must be installed in that same environment.

## Request and response

Protocol version 1 has one operation. Paths in requests are absolute, normalized filesystem paths. The graph selector is explicit and uses dotted attribute lookup.

Example request (also saved as [`request-v1.json`](fixtures/metadata-extraction/request-v1.json)):

```json
{
  "protocol_version": 1,
  "operation": "extract_graph_info",
  "source_file": "/workspace/docs/fixtures/metadata-extraction/graph.py",
  "graph_object": "graph"
}
```

On success the helper writes exactly one JSON document to stdout. `graph_info` is the JSON representation returned by the existing `GraphInfo` serializer; its field names and nullable fields are not independently redefined by this protocol.

Example response (also saved as [`response-v1.json`](fixtures/metadata-extraction/response-v1.json)):

```json
{
  "protocol_version": 1,
  "graph_info": {
    "nodes": [
      {
        "name": "start",
        "func_name": "start_node",
        "signature": "(state)",
        "docstring": null,
        "source_hint": "/workspace/docs/fixtures/metadata-extraction/graph.py:4"
      }
    ],
    "edges": [
      { "from": "start", "to": "__end__", "condition": null }
    ],
    "state_schema": {
      "fields": [
        {
          "name": "count",
          "type_name": "int",
          "is_optional": false,
          "default_value": null
        }
      ]
    },
    "entry_point": "start",
    "conditional_edges": {}
  }
}
```

The response wrapper's `protocol_version` is separate from the `GraphInfo` shape. The CLI validates the version, then deserializes `graph_info` as `GraphInfo`. The fixture mirrors the existing `extracts_graph_metadata_with_current_python_api` unit fixture, including the function's line number; the absolute `source_hint` path is illustrative.

## Output, diagnostics, and errors

- On success, stdout contains only the response JSON and the helper exits with status 0. User module stdout is redirected to stderr so it cannot corrupt the protocol stream.
- On failure, stdout is empty, the helper exits nonzero, and stderr contains a concise diagnostic. A traceback is not part of the protocol; an explicit future debug option may add one.
- The CLI captures stdout and stderr separately. It rejects empty output, multiple JSON values, malformed JSON, an unsupported response version, or a response without a `graph_info` object.
- Invalid request JSON, missing or unknown request fields, fields with the wrong type, an unsupported protocol version or operation, a nonexistent/non-file source path, an invalid selector, an interpreter launch error, an import error, a missing selected object, an extraction error, or invalid response JSON fails without producing graph output.
- The CLI enforces a 30-second default timeout. `--timeout` accepts a positive duration to override it. On expiry it terminates the helper and reports a timeout on stderr; partial stdout is discarded.
- The process exit status, not a JSON error envelope, represents failure. This keeps stdout reserved for one success document and avoids a second error schema.

## Execution and portability boundary

Loading a user file executes its top-level Python code, including imports and any side effects. This is arbitrary code execution with the selected interpreter's filesystem, environment, network access, and credentials. Users should run extraction only on files they trust and should choose an isolated Python environment when needed. The CLI does not claim to sandbox Python.

Interpreter discovery and helper launch are lazy and limited to extraction commands. `--help`, `--version`, and their subcommand help must continue to work when Python is absent. The CLI must continue to depend on `langgraph-inspector` with `default-features = false`; enabling its `python` feature in the CLI would restore the link-time dependency this adapter is intended to avoid.

## Source facts checked on main

At `867d5beef348616b433fa029e335e247940b21f3`:

- `langgraph-cli/Cargo.toml` sets `default-features = false` for `langgraph-inspector`.
- `langgraph-inspector/src/lib.rs` defines `GraphInfo`, `NodeInfo`, `EdgeInfo`, `StateSchema`, `FieldInfo`, and `ConditionalEdge`. `GraphInfo` has the fields used by the response fixture.
- `langgraph-inspector/src/python.rs` exports `extract_graph_info`, serializes `GraphInfo` to JSON, and contains the `SimpleNamespace` extraction test mirrored by the fixture.
- `langgraph-cli/src/main.rs` still reports that Python graph extraction is not implemented; this proposal does not change that behavior.
- `langgraph-cli/tests/cli_contract.rs` checks help/version and current placeholder behavior. `.github/workflows/release.yml` smoke-tests the built binary with `--help` and `--version`, and on macOS inspects linked libraries with `otool -L`.

These facts support the boundary and shape of the proposal. They do not establish that the helper extension is currently packaged for external Python interpreters; that packaging decision is explicitly left for review before implementation.
