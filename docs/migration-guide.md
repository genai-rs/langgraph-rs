# Migration Guide: Python LangGraph to Rust

This guide helps you migrate existing LangGraph Python workflows to high-performance Rust implementations using langgraph-rs.

## Overview

langgraph-rs converts your Python LangGraph workflows into equivalent Rust code that runs 5-10x faster with significantly lower memory usage.

## Quick Migration

### Step 1: Install langgraph-rs

```bash
cargo install langgraph-rs
```

### Step 2: Convert Your Workflow

```bash
langgraph-rs convert my_workflow.py --output ./rust_graph/
```

### Step 3: Review Generated Code

```bash
cd rust_graph
cat src/lib.rs  # Review generated code
```

### Step 4: Implement Node Logic

Replace TODO stubs with actual implementations:

```rust
// Before (generated stub)
async fn llm_node(mut state: GraphState) -> Result<GraphState> {
    // TODO: Implement llm logic here
    Ok(state)
}

// After (your implementation)
async fn llm_node(mut state: GraphState) -> Result<GraphState> {
    let llm = OpenAIProvider::new(api_key, "gpt-4".to_string());
    let prompt = format!("Process: {}", state.input);
    let response = llm.complete(&prompt).await?;
    state.output = response;
    Ok(state)
}
```

### Step 5: Test

```bash
cargo test
cargo run
```

## Detailed Migration

### State Schema Migration

**Python:**
```python
from typing import TypedDict

class AgentState(TypedDict):
    messages: list[str]
    context: dict
    counter: int
```

**Generated Rust:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {
    pub messages: Vec<String>,
    pub context: HashMap<String, serde_json::Value>,
    pub counter: i64,
}
```

**Type Mappings:**

| Python Type | Rust Type | Notes |
|------------|-----------|-------|
| `str` | `String` | Always owned |
| `int` | `i64` | 64-bit signed |
| `float` | `f64` | Double precision |
| `bool` | `bool` | Same |
| `list[T]` | `Vec<T>` | Dynamic array |
| `dict[K, V]` | `HashMap<K, V>` | Hash map |
| `Optional[T]` | `Option<T>` | Nullable |
| `Any` | `serde_json::Value` | Fallback |

### Node Function Migration

**Python:**
```python
def process_node(state: AgentState) -> AgentState:
    """Process user input."""
    state["messages"].append("Processing...")
    state["counter"] += 1
    return state
```

**Rust:**
```rust
/// Process user input.
async fn process_node(mut state: GraphState) -> Result<GraphState> {
    state.messages.push("Processing...".to_string());
    state.counter += 1;
    Ok(state)
}
```

**Key Differences:**
1. **Async**: All node functions are async in Rust
2. **Error Handling**: Return `Result<GraphState>` instead of just `GraphState`
3. **Mutability**: Use `mut` keyword for mutable state
4. **Ownership**: Be aware of Rust's ownership rules

### Edge Migration

**Python:**
```python
workflow.add_edge("start", "process")
workflow.add_conditional_edges(
    "process",
    route_next,
    {
        "continue": "process",
        "end": END
    }
)
```

**Rust (generated):**
```rust
// In execute_graph function
match current_node {
    "process" => {
        state = process_node(state).await?;
        current_node = match process_route(&state) {
            "continue" => "process",
            "end" => break,
            _ => bail!("Unknown route"),
        };
    }
    // ...
}
```

### LLM Integration Migration

**Python:**
```python
from langchain.llms import OpenAI

def llm_node(state: AgentState) -> AgentState:
    llm = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
    response = llm.complete(state["input"])
    state["output"] = response
    return state
```

**Rust:**
```rust
use langgraph_runtime::OpenAIProvider;

async fn llm_node(mut state: GraphState) -> Result<GraphState> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let llm = OpenAIProvider::new(api_key, "gpt-4".to_string());

    let response = llm.complete(&state.input).await?;
    state.output = response;

    Ok(state)
}
```

### Tool Integration Migration

**Python:**
```python
from langchain.tools import Tool

search_tool = Tool(
    name="search",
    func=lambda q: search_api(q),
    description="Search the web"
)

def tool_node(state: AgentState) -> AgentState:
    result = search_tool.run(state["query"])
    state["results"] = result
    return state
```

**Rust:**
```rust
use langgraph_runtime::Tool;

struct SearchTool {
    client: reqwest::Client,
}

#[async_trait]
impl Tool for SearchTool {
    async fn invoke(&self, input: Value) -> Result<Value> {
        let query = input.as_str().unwrap();
        // Implement search logic
        Ok(json!({"results": "..."}))
    }

    fn name(&self) -> &str { "search" }
    fn description(&self) -> &str { "Search the web" }
}

async fn tool_node(mut state: GraphState) -> Result<GraphState> {
    let tool = SearchTool { client: reqwest::Client::new() };
    let result = tool.invoke(json!(state.query)).await?;
    state.results = serde_json::from_value(result)?;
    Ok(state)
}
```

## Common Patterns

### Pattern 1: Simple Linear Workflow

**Python:**
```python
workflow = StateGraph(State)
workflow.add_node("a", node_a)
workflow.add_node("b", node_b)
workflow.add_edge("a", "b")
workflow.set_entry_point("a")
```

**Migration:** Fully automatic, no manual changes needed.

### Pattern 2: Conditional Branching

**Python:**
```python
def router(state: State) -> str:
    return "high" if state["value"] > 50 else "low"

workflow.add_conditional_edges("start", router, {
    "high": "high_path",
    "low": "low_path"
})
```

**Rust (implement router):**
```rust
fn start_route(state: &GraphState) -> &'static str {
    if state.value > 50 {
        "high"
    } else {
        "low"
    }
}
```

### Pattern 3: Looping

**Python:**
```python
def should_continue(state: State) -> str:
    if state["counter"] < 5:
        return "continue"
    return "end"

workflow.add_conditional_edges("loop", should_continue, {
    "continue": "loop",
    "end": END
})
```

**Rust (implement condition):**
```rust
fn loop_route(state: &GraphState) -> &'static str {
    if state.counter < 5 {
        "continue"
    } else {
        "end"
    }
}
```

## Testing Migration

### Unit Tests

**Python:**
```python
def test_node():
    state = {"value": 0}
    result = process_node(state)
    assert result["value"] == 1
```

**Rust:**
```rust
#[tokio::test]
async fn test_node() {
    let mut state = GraphState::new();
    state.value = 0;

    let result = process_node(state).await.unwrap();
    assert_eq!(result.value, 1);
}
```

### Integration Tests

Use the validation framework:

```bash
# Compare Python and Rust outputs
python tests/validation/compare_outputs.py my_workflow
```

## Performance Tuning

After migration, optimize for production:

1. **Enable Release Mode:**
   ```bash
   cargo build --release
   ```

2. **Profile:**
   ```bash
   cargo flamegraph --bin my_workflow
   ```

3. **Optimize Hot Paths:**
   - Cache repeated computations
   - Use connection pooling for LLM calls
   - Batch operations where possible

4. **Benchmark:**
   ```bash
   cargo bench
   ```

## Troubleshooting

### Issue: Type Mismatch

**Error:** `expected HashMap<String, Value>, found HashMap<String, String>`

**Solution:** Update type annotations in Python or manually adjust Rust types.

### Issue: Lifetime Errors

**Error:** `borrowed value does not live long enough`

**Solution:** Clone values or restructure code to avoid borrowing issues.

```rust
// Instead of:
let x = &state.value;  // Borrow

// Use:
let x = state.value.clone();  // Clone
```

### Issue: Async/Await

**Error:** `cannot await in non-async function`

**Solution:** Ensure all node functions are async and use `.await`:

```rust
async fn node(state: GraphState) -> Result<GraphState> {
    let result = async_operation().await?;  // Note the .await
    Ok(state)
}
```

## Best Practices

1. **Start Small:** Migrate simple workflows first
2. **Test Incrementally:** Test each node individually
3. **Use Validation:** Compare Python and Rust outputs
4. **Document Changes:** Note any semantic differences
5. **Benchmark:** Measure performance improvements
6. **Monitor:** Set up logging and monitoring

## Migration Checklist

- [ ] Convert workflow with langgraph-rs
- [ ] Review generated code
- [ ] Implement node logic
- [ ] Implement routing functions
- [ ] Add error handling
- [ ] Write unit tests
- [ ] Run validation tests
- [ ] Benchmark performance
- [ ] Deploy to staging
- [ ] Monitor in production

## Getting Help

- **Documentation:** https://docs.langgraph-rs.io
- **Examples:** See `examples/` directory
- **Issues:** https://github.com/genai-rs/langgraph-rs/issues
- **Email:** tim.van.wassenhove@gmail.com

## Next Steps

After successful migration:

1. Review [Performance Guide](performance.md)
2. Set up [Deployment](deployment.md)
3. Implement [Monitoring](monitoring.md)
4. Share your success story!
