"""Workflow with explicit looping behavior."""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class LoopState(TypedDict):
    """State for tracking loop iterations."""
    iteration: int
    max_iterations: int
    accumulated_value: int
    history: list[str]


def initialize(state: LoopState) -> LoopState:
    """Set up initial state."""
    state["iteration"] = 0
    state["accumulated_value"] = 0
    state["history"].append(f"Initialized")
    return state


def loop_body(state: LoopState) -> LoopState:
    """Execute loop iteration."""
    state["iteration"] += 1
    state["accumulated_value"] += state["iteration"]
    state["history"].append(f"Iteration {state['iteration']}: accumulated={state['accumulated_value']}")
    return state


def check_continue(state: LoopState) -> Literal["loop_body", "finalize"]:
    """Decide whether to continue looping."""
    if state["iteration"] < state["max_iterations"]:
        return "loop_body"
    else:
        return "finalize"


def finalize(state: LoopState) -> LoopState:
    """Cleanup and final calculations."""
    state["history"].append(f"Completed {state['iteration']} iterations")
    return state


# Build the graph
workflow = StateGraph(LoopState)

# Add nodes
workflow.add_node("initialize", initialize)
workflow.add_node("loop_body", loop_body)
workflow.add_node("finalize", finalize)

# Set entry point
workflow.set_entry_point("initialize")

# Add edges
workflow.add_edge("initialize", "loop_body")

# Conditional loop back
workflow.add_conditional_edges(
    "loop_body",
    check_continue,
    {
        "loop_body": "loop_body",  # Loop back to self
        "finalize": "finalize"
    }
)

workflow.add_edge("finalize", END)

# Compile the graph
graph = workflow.compile()


if __name__ == "__main__":
    initial_state = {
        "iteration": 0,
        "max_iterations": 5,
        "accumulated_value": 0,
        "history": []
    }

    result = graph.invoke(initial_state)
    print(f"Completed {result['iteration']} iterations")
    print(f"Final accumulated value: {result['accumulated_value']}")
    print("\nHistory:")
    for entry in result['history']:
        print(f"  {entry}")
