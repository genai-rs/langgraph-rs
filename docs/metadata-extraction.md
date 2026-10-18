# Python metadata extraction protocol (proposal)

**Status:** proposal amended from exact-head review. This document specifies an interface; it does not enable Python execution in the CLI.

## Decision and package layout

Use a subprocess adapter for `inspect` and the extraction step used by `convert`. The adapter keeps the Rust CLI independent of CPython at link and startup time. It loads the user's file with the selected Python interpreter, then calls the existing `langgraph_inspector.extract_graph_info` PyO3 function. It must not reproduce that function's graph traversal in Python or translate arbitrary Python code into Rust.

Package the helper and existing extension locally under `langgraph-inspector`, leaving the root Python dependency project alone:

```text
langgraph-inspector/
  pyproject.toml
  python/langgraph_inspector/
    __init__.py
    helper.py
```

`langgraph-inspector/pyproject.toml` uses the Maturin PEP 517 backend (`maturin>=1.9.4,<2`), with `manifest-path = "Cargo.toml"`, `python-source = "python"`, `module-name = "langgraph_inspector.langgraph_inspector"`, and the inspector's `python` feature enabled for this extension build. This matches the current PyO3 0.29.3 setup. The distribution is named `langgraph-inspector`. The native module maps to the existing `PyInit_langgraph_inspector` symbol from `#[pymodule] fn langgraph_inspector`; no Rust traversal or symbol rename is needed. `python/langgraph_inspector/__init__.py` re-exports `extract_graph_info` from `.langgraph_inspector`. `helper.py` calls that function and parses its returned JSON text before placing the value in the protocol response.

Users install the local source package into the selected virtual environment explicitly with `<selected-python> -m pip install ./langgraph-inspector` from the repository root. Source installation requires Rust. The CLI does not install dependencies automatically. This design adds no wheel publication or release workflow changes. Maturin sets the extension build mode for the current PyO3 version; do not add the deprecated `extension-module` Cargo feature globally. Keep the CLI's inspector dependency at `default-features = false`.

## Invocation and interpreter selection

An extraction command supplies a Python file and a required graph object selector:

```text
langgraph-rs inspect examples/workflow.py --graph-object graph
langgraph-rs inspect examples/workflow.py --graph-object workflow.graph \
  --python /path/to/venv/bin/python --timeout 45s
```

`--graph-object` is a dotted sequence of Python identifiers resolved by attribute lookup. It does not accept expressions, calls, indexing, or an inferred default. The command fails if the selector is absent, malformed, or resolves to no object. The helper does not call `compile()` on the selected object; it passes it to the existing extractor as-is.

Choose the interpreter in this order:

1. An explicit `--python` executable path.
2. `LANGGRAPH_PYTHON`, if set to an executable path.
3. `python3` found on `PATH` (with `python` as the Windows fallback).

If the explicit option or environment variable is set but cannot be launched, report that failure; do not silently fall back to another interpreter. Do not evaluate shell syntax or install dependencies. The CLI starts the helper only when an extraction operation runs. It does not probe Python while parsing arguments or handling `--help` and `--version`.

The CLI invokes `<selected-python> -m langgraph_inspector.helper` and sends one UTF-8 JSON request on stdin. The package and extension must be importable by the chosen interpreter, and LangGraph dependencies used by the file must be installed in that same environment.

## Source loading and graph selection

The helper requires `source_file` to resolve to a normalized absolute path for an existing regular file. It executes the file once as a synthetic module using `importlib`, with an explicit module name registered in `sys.modules` before execution. It preserves the caller's working directory and temporarily prepends the source file's parent to `sys.path` so sibling imports work. Package-relative imports are outside this initial single-file contract and must produce a diagnostic; the helper must not guess a package root or execute the file twice.

The selector is split on dots, and every component must be a Python identifier. Resolve the first component from the loaded module namespace, then resolve remaining components with `getattr`. Do not use `eval`, calls, indexing, or graph auto-detection. Missing attributes fail explicitly. Imports and attribute access can execute user code.

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

## Output, diagnostics, and bounded completion

- Before importing user code or extracting metadata, the helper flushes Python streams, duplicates protocol stdout to a saved non-inheritable file descriptor, then redirects descriptor 1 to descriptor 2 for the rest of the helper lifetime. This isolates the protocol from `print`, `os.write(1, ...)`, and ordinary native writes by imported code. The helper writes the final JSON only through the saved protocol descriptor, and only after extraction and response serialization succeed. This is protocol isolation, not a hostile-code sandbox.
- On success, the saved protocol descriptor contains exactly one response JSON document and the helper exits with status 0. Extraction or serialization failures write no protocol bytes; the helper exits nonzero and stderr contains a concise diagnostic. If the final pipe write itself fails, the CLI treats partial output as failure and discards it. A traceback is not part of the protocol; an explicit future debug option may add one.
- The CLI starts draining stdout and stderr concurrently when it launches the child, and closes stdin after sending the single request. It caps captured stdout at 16 MiB; exceeding the cap fails the operation and terminates the direct helper. It retains only the most recent 1 MiB of stderr for diagnostics while continuing to drain stderr so the child cannot block on a full pipe.
- One 30-second default deadline covers request writing, helper execution, and output collection. `--timeout` accepts a positive duration to override it. On expiry, the CLI terminates, kills if needed, and reaps the direct helper within a fixed 1-second cleanup grace, then discards partial stdout. It cancels and closes pipe readers after that grace even if a descendant still holds a pipe open. The CLI returns after the deadline and cleanup grace rather than waiting indefinitely for EOF.
- If the helper exits successfully, the CLI rejects empty output, multiple JSON values, malformed JSON, unknown response fields, an unsupported response version, a missing `graph_info` object, or a `graph_info` value that cannot deserialize as `GraphInfo`.
- Request and response version 1 reject unknown fields and fields with the wrong type. Invalid request JSON, an unsupported protocol version or operation, a nonexistent/non-file source path, an invalid selector, an interpreter launch error, an import error, a missing selected object, or an extraction error fails without producing graph output.
- The process exit status, not a JSON error envelope, represents failure. This keeps stdout reserved for one success document and avoids a second error schema.
- These process guarantees cover the direct helper and the CLI's return. User code can create descendants and side effects; the CLI does not promise to stop descendants, contain them, or sandbox their access.

## Execution and portability boundary

Loading a user file executes its top-level Python code, imports, and selected-object attribute access, including side effects. This is arbitrary code execution with the selected interpreter's filesystem, environment, network access, and credentials. Users should run extraction only on files they trust and should choose an isolated Python environment when needed. The CLI does not claim to sandbox Python or terminate arbitrary descendants.

Interpreter discovery and helper launch are lazy and limited to extraction commands. `--help`, `--version`, and their subcommand help must continue to work when Python is absent. The CLI must continue to depend on `langgraph-inspector` with `default-features = false`; enabling its `python` feature in the CLI would restore the link-time dependency this adapter is intended to avoid.

## Adapter implementation boundary and acceptance

The follow-on implementation is bounded to the local mixed-package metadata above and one shared Rust subprocess adapter reused by `inspect` and `convert`. If visualization extraction is enabled, route it through that same adapter; otherwise leave it unimplemented. Do not add a second graph traversal, Python-to-Rust translation, tracing, validation, automatic installation, wheel publication, or release workflow changes.

Before accepting the adapter, verify all of the following:

1. Install `./langgraph-inspector` into a clean selected virtual environment and run the packaged helper from a working directory outside the repository.
2. Send the documented request for both `docs/fixtures/metadata-extraction/graph.py` and the existing `tests/fixtures/linear_workflow.py`. Parse the actual response and deserialize its `graph_info` as `GraphInfo`; compare the synthetic fixture after normalizing the absolute `source_hint` path.
3. Exercise malformed JSON, wrong or unknown fields, unsupported versions, missing files and selectors, import errors, invalid explicit interpreters (with no fallback), nonzero helper exits, stdout writes from Python and native code, stdout over 16 MiB, stderr over 1 MiB, and timeout while output readers are active.
4. Confirm the CLI returns after bounded direct-child cleanup even when a descendant retains a pipe. The test harness must clean up any descendant it creates; the CLI makes no process-tree termination guarantee.
5. Run `--help` and `--version` with no usable Python on `PATH` and verify no interpreter is launched. Preserve the existing release portability checks.

LG03 remains a documentation and fixture proposal until this interface is accepted. Do not implement the adapter as part of this proposal.

## Source facts checked on main

At `867d5beef348616b433fa029e335e247940b21f3`:

- `langgraph-cli/Cargo.toml` sets `default-features = false` for `langgraph-inspector`.
- `langgraph-inspector/src/lib.rs` defines `GraphInfo`, `NodeInfo`, `EdgeInfo`, `StateSchema`, `FieldInfo`, and `ConditionalEdge`. `GraphInfo` has the fields used by the response fixture.
- `langgraph-inspector/src/python.rs` exports `extract_graph_info`, serializes `GraphInfo` to JSON, and contains the `SimpleNamespace` extraction test mirrored by the fixture.
- `langgraph-cli/src/main.rs` still reports that Python graph extraction is not implemented; this proposal does not change that behavior.
- `langgraph-cli/tests/cli_contract.rs` checks help/version and current placeholder behavior. `.github/workflows/release.yml` smoke-tests the built binary with `--help` and `--version`, and on macOS inspects linked libraries with `otool -L`. The workflow installs Python for other build checks, so those smoke tests alone do not prove Python-absent startup; the explicit Python-absent probe above is required for adapter acceptance.

These facts support the boundary and shape of the proposal. The proposed extension package and helper do not exist yet; they are part of the bounded follow-on adapter work, not a claim about current packaging.
