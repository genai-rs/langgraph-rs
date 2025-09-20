"""Loop workflow test fixture - iterative processing."""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class LoopState(TypedDict):
    """State for loop workflow."""
    counter: int
    accumulator: int
    history: list[str]
    max_iterations: int


def increment(state: LoopState) -> LoopState:
    """Increment counter and accumulate."""
    state["counter"] += 1
    state["accumulator"] += state["counter"]
    state["history"].append(f"Iteration {state['counter']}: sum = {state['accumulator']}")
    return state


def should_continue(state: LoopState) -> Literal["increment", "done"]:
    """Decide whether to continue looping."""
    if state["counter"] < state["max_iterations"]:
        return "increment"
    else:
        return "done"


def finalize_loop(state: LoopState) -> LoopState:
    """Finalize the loop results."""
    state["history"].append(f"Final result: {state['accumulator']}")
    return state


# Build the graph
workflow = StateGraph(LoopState)

# Add nodes
workflow.add_node("increment", increment)
workflow.add_node("finalize", finalize_loop)

# Set entry point
workflow.set_entry_point("increment")

# Add conditional edges for looping
workflow.add_conditional_edges(
    "increment",
    should_continue,
    {
        "increment": "increment",  # Loop back
        "done": "finalize"
    }
)

# Add terminal edge
workflow.add_edge("finalize", END)

# Compile
graph = workflow.compile()