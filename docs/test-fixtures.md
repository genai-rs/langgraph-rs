# Test Fixtures Documentation

This document describes the test fixtures used for LangGraph introspection and conversion testing in langgraph-rs.

## Overview

Test fixtures are located in `tests/fixtures/` and provide representative examples of various LangGraph patterns. These fixtures serve multiple purposes:

1. **Introspection Testing**: Verify that PyO3 can correctly extract metadata from LangGraph instances
2. **Conversion Validation**: Ensure generated Rust code matches expected patterns
3. **Integration Testing**: Compare Python and Rust execution outputs
4. **Documentation**: Serve as examples of supported LangGraph features

## Fixture Categories

### 1. Linear Workflow (`linear_workflow.py`)

**Purpose**: Test simple sequential node execution without branching.

**Key Features**:
- Simple state with basic types (int, list[str])
- Three sequential nodes: A → B → C
- No conditional logic
- Direct edges only

**Expected Introspection Output**:
```rust
GraphInfo {
    nodes: vec!["a", "b", "c"],
    edges: vec![
        Edge { from: "a", to: "b", condition: None },
        Edge { from: "b", to: "c", condition: None },
        Edge { from: "c", to: END, condition: None },
    ],
    state_schema: StateSchema {
        fields: vec![
            Field { name: "counter", type: "i64" },
            Field { name: "messages", type: "Vec<String>" },
        ]
    }
}
```

**Test Expectations**:
- Counter should increment 3 times (final value: 3)
- Messages list should contain 3 entries
- Execution should be deterministic

### 2. Conditional Workflow (`conditional_workflow.py`)

**Purpose**: Test conditional branching and routing logic.

**Key Features**:
- State with conditional fields
- Two processing paths (high/low) based on value threshold
- Loop-back capability
- Multiple conditional edge functions

**Expected Introspection Output**:
```rust
GraphInfo {
    nodes: vec!["start", "high", "low", "end"],
    conditional_edges: vec![
        ConditionalEdge {
            from: "start",
            router: "route_based_on_value",
            mapping: {
                "high": "high",
                "low": "low"
            }
        },
        ConditionalEdge {
            from: "high",
            router: "should_continue",
            mapping: {
                "start": "start",
                "end": "end"
            }
        },
        // ...
    ]
}
```

**Test Expectations**:
- Different execution paths based on initial value
- Loop termination when value >= 100
- Path tracking in state

### 3. Complex Types (`complex_types.py`)

**Purpose**: Test handling of nested and complex Python types.

**Key Features**:
- Nested dictionaries and lists
- Optional types
- Mixed type dictionaries (dict[str, any])
- Custom dataclasses
- Type-specific operations

**Expected Type Mappings**:
```rust
ComplexState {
    users: Vec<HashMap<String, serde_json::Value>>,
    metadata: HashMap<String, serde_json::Value>,
    current_user: Option<HashMap<String, serde_json::Value>>,
    scores: HashMap<String, f64>,
    flags: HashMap<String, bool>,
}
```

**Test Expectations**:
- Proper serialization/deserialization of nested structures
- None/null handling for optional fields
- Type preservation through conversion

### 4. Loop Workflow (`loop_workflow.py`)

**Purpose**: Test explicit loop patterns with iteration tracking.

**Key Features**:
- Self-referencing conditional edges
- Iteration counter
- Accumulated state across iterations
- Loop termination condition

**Expected Behavior**:
- Fixed number of iterations (configurable via state)
- State accumulation (sum of 1..N)
- History tracking

**Test Expectations**:
- For max_iterations=5: accumulated_value should be 15 (1+2+3+4+5)
- History should have exactly 7 entries (init + 5 iterations + finalize)
- No infinite loops

### 5. Multi-Path Workflow (`multi_path_workflow.py`)

**Purpose**: Test multiple independent paths that converge.

**Key Features**:
- Three separate processing paths (A, B, C)
- Input-based routing
- Path convergence (all paths merge to same node)
- Independent path results

**Expected Graph Structure**:
```
start → router → [path_a, path_b, path_c] → merge → END
```

**Test Expectations**:
- Only one path executes per invocation
- All paths converge to merge node
- Path selection is deterministic based on input

## Running the Fixtures

### Prerequisites

```bash
# Set up Python environment
./scripts/setup-python.sh

# Activate virtual environment
source .venv/bin/activate
```

### Execute Individual Fixtures

```bash
# Run a specific fixture
python tests/fixtures/linear_workflow.py

# Expected output format
Final counter: 3
Messages: ['Node A executed', 'Node B executed', 'Node C executed']
```

### Run All Fixtures

```bash
# Execute all fixtures to verify they work
for fixture in tests/fixtures/*.py; do
    echo "Running $fixture..."
    python "$fixture"
done
```

## Fixture Requirements

All fixtures must adhere to the following requirements:

### 1. Structure Requirements

- **Must** use `from langgraph.graph import StateGraph, END`
- **Must** define a TypedDict for state
- **Must** compile the graph with `.compile()`
- **Should** include a `__main__` block for manual testing

### 2. State Requirements

- All state fields must have type annotations
- Use standard Python types (str, int, float, bool, list, dict)
- Complex types should use typing module (Optional, Literal, etc.)
- Avoid unnecessary Any types

### 3. Node Function Requirements

- Functions must accept state as parameter
- Functions must return state (same type)
- Functions should have descriptive docstrings
- Functions should be pure (no side effects) when possible

### 4. Documentation Requirements

- Each fixture file must have a module docstring
- Each node function must have a docstring
- Complex routing logic should be commented

### 5. Testing Requirements

- Fixtures must be executable standalone
- Output must be deterministic (for same input)
- Should complete in < 1 second
- Should not require external dependencies (APIs, databases)

## Adding New Fixtures

To add a new test fixture:

1. **Create the fixture file** in `tests/fixtures/`
2. **Follow the naming convention**: `<pattern>_workflow.py`
3. **Implement the pattern** following requirements above
4. **Test manually**: `python tests/fixtures/<your_fixture>.py`
5. **Document the fixture** in this file
6. **Add integration test** in `tests/test_introspection.py`

### Example Template

```python
"""Brief description of this workflow pattern."""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class YourState(TypedDict):
    """State schema description."""
    field1: str
    field2: int


def node_function(state: YourState) -> YourState:
    """What this node does."""
    # Implementation
    return state


# Build the graph
workflow = StateGraph(YourState)
workflow.add_node("node1", node_function)
workflow.set_entry_point("node1")
workflow.add_edge("node1", END)
graph = workflow.compile()


if __name__ == "__main__":
    initial_state = {"field1": "value", "field2": 0}
    result = graph.invoke(initial_state)
    print(f"Result: {result}")
```

## Fixture Validation

Before submitting a new fixture, validate it meets all requirements:

```bash
# Check Python syntax and types
mypy tests/fixtures/<your_fixture>.py

# Format code
black tests/fixtures/<your_fixture>.py

# Lint code
ruff tests/fixtures/<your_fixture>.py

# Run the fixture
python tests/fixtures/<your_fixture>.py
```

## Integration with Rust Tests

These Python fixtures are used by Rust integration tests to:

1. **Extract metadata** via PyO3 introspection
2. **Generate Rust code** from the extracted metadata
3. **Compare outputs** between Python and Rust execution
4. **Validate performance** improvements

See `langgraph-inspector/tests/` for Rust test implementations that use these fixtures.

## Troubleshooting

### Common Issues

**Problem**: `ImportError: No module named 'langgraph'`
- **Solution**: Activate virtual environment and install dependencies

**Problem**: Fixture hangs or times out
- **Solution**: Check for infinite loops in conditional edges

**Problem**: Non-deterministic output
- **Solution**: Remove randomness, use fixed seeds, or avoid time-dependent logic

**Problem**: Type errors during introspection
- **Solution**: Ensure all fields have proper type annotations

## Future Fixture Categories

Planned fixtures for future development:

- **tool_workflow.py**: LangGraph with tool/function calling
- **async_workflow.py**: Async node execution patterns
- **streaming_workflow.py**: Streaming response handling
- **parallel_workflow.py**: Parallel node execution
- **subgraph_workflow.py**: Nested graph patterns
- **human_in_loop_workflow.py**: Workflows with human feedback

## References

- [LangGraph Documentation](https://python.langchain.com/docs/langgraph)
- [LangGraph Concepts](https://langchain-ai.github.io/langgraph/concepts/)
- [TypedDict Documentation](https://docs.python.org/3/library/typing.html#typing.TypedDict)
- [PyO3 Documentation](https://pyo3.rs/)
