#***REMOVED*** AI Assistant Instructions

## Project: langgraph-rs

This project converts LangGraph (Python) workflows into high-performance Rust code for production LLM applications.

## Primary Instructions

Please refer to the comprehensive agent instructions at:
- [AGENTS.md](./AGENTS.md) - Detailed development guidelines and project context

## Quick Context

- **Project Location**: `/Users/tim.van.wassenhove/src/github/langgraph-rs`
- **Current Phase**: Phase 2 - Python Introspection (See TODO.md)
- **Main Goal**: Build a transpiler from LangGraph (Python) to Rust

## Key Files to Review

1. **[AGENTS.md](./AGENTS.md)** - Complete agent instructions
2. **[TODO.md](./TODO.md)** - Current development priorities
3. **[README.md](./README.md)** - Project overview
4. **examples/simple_workflow.py** - Example LangGraph workflow
5. **examples/simple_workflow_generated.rs** - Target Rust output

## Current Focus Areas

Based on TODO.md, we're currently working on:
- Python environment setup
- PyO3 introspection implementation
- Type mapping system
- Graph edge extraction

## How to Help

1. **Read AGENTS.md first** - Contains all architectural decisions and guidelines
2. **Check TODO.md** - See what needs to be done
3. **Follow the patterns** - Use existing code as templates
4. **Test everything** - We aim for 90%+ coverage
5. **Document changes** - Update relevant documentation

## Quick Commands

```bash
# Build the project
cargo build

# Run the CLI
cargo run --bin langgraph-rs -- --help

# Run tests
cargo test

# Check code quality
cargo clippy
```

## Remember

- This is a performance-focused project (5-10x improvement goal)
- Generated code should be production-ready
- Follow Rust idioms and best practices
- Preserve Python semantics in the conversion

For detailed instructions, please see **[AGENTS.md](./AGENTS.md)**