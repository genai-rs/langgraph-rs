# Performance

## Current evidence

The project does not currently have a reproducible end-to-end benchmark comparing equivalent Python and Rust LangGraph workflows. Python graph extraction and behavioral validation are unimplemented, and generated node functions are placeholders. The previously published latency, throughput, memory, and speedup figures have therefore been removed; they are not supported by benchmark evidence.

The source file [`tests/benchmarks/benchmark_conversion.rs`](../tests/benchmarks/benchmark_conversion.rs) contains timing checks for generating code from synthetic graph metadata with 5, 20, and 100 nodes. These are ordinary Rust `#[test]` functions with generous time limits. They do not compare Python with Rust, execute generated workflow logic, or establish an application performance guarantee. The file's location under `tests/benchmarks/` does not make these measurements a Cargo benchmark target.

## How to evaluate performance later

Meaningful performance claims require equivalent implementations and a reproducible benchmark. Once graph conversion and execution have working semantics, record:

- the repository commit, benchmark command, and benchmark source;
- the machine, operating system, Rust compiler, and Python versions;
- the workflow inputs and node behavior, including how external LLM or network calls are handled;
- separate measurements for conversion time and workflow execution, with repeated runs and warm-up;
- latency and throughput results, and memory results if memory improvements are claimed.

Compare implementations on the same workload and report the measurement method and variation. Do not use synthetic code-generation timings as a proxy for workflow execution performance.
