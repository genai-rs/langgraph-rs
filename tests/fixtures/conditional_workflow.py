"""Conditional workflow with branching logic."""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class ConditionalState(TypedDict):
    """State with condition tracking."""
    value: int
    path_taken: list[str]
    should_continue: bool


def start_node(state: ConditionalState) -> ConditionalState:
    """Initialize the workflow."""
    state["path_taken"].append("start")
    return state


def process_high(state: ConditionalState) -> ConditionalState:
    """Process for high values."""
    state["path_taken"].append("high")
    state["value"] *= 2
    state["should_continue"] = state["value"] < 100
    return state


def process_low(state: ConditionalState) -> ConditionalState:
    """Process for low values."""
    state["path_taken"].append("low")
    state["value"] += 10
    state["should_continue"] = state["value"] < 100
    return state


def end_node(state: ConditionalState) -> ConditionalState:
    """Finalize the workflow."""
    state["path_taken"].append("end")
    return state


def route_based_on_value(state: ConditionalState) -> Literal["high", "low"]:
    """Route based on value threshold."""
    return "high" if state["value"] >= 50 else "low"


def should_continue(state: ConditionalState) -> Literal["start", "end"]:
    """Determine if we should loop or finish."""
    return "start" if state["should_continue"] else "end"


# Build the graph
workflow = StateGraph(ConditionalState)

# Add nodes
workflow.add_node("start", start_node)
workflow.add_node("high", process_high)
workflow.add_node("low", process_low)
workflow.add_node("end", end_node)

# Set entry point
workflow.set_entry_point("start")

# Add conditional routing
workflow.add_conditional_edges(
    "start",
    route_based_on_value,
    {
        "high": "high",
        "low": "low"
    }
)

# Both paths can loop back or end
workflow.add_conditional_edges(
    "high",
    should_continue,
    {
        "start": "start",
        "end": "end"
    }
)

workflow.add_conditional_edges(
    "low",
    should_continue,
    {
        "start": "start",
        "end": "end"
    }
)

workflow.add_edge("end", END)

# Compile the graph
graph = workflow.compile()


if __name__ == "__main__":
    initial_state = {"value": 25, "path_taken": [], "should_continue": True}
    result = graph.invoke(initial_state)
    print(f"Final value: {result['value']}")
    print(f"Path taken: {' -> '.join(result['path_taken'])}")
