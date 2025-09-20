"""Conditional workflow test fixture - branching logic."""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class ConditionalState(TypedDict):
    """State for conditional workflow."""
    value: int
    path_taken: str
    result: str


def check_value(state: ConditionalState) -> ConditionalState:
    """Check the input value."""
    if state["value"] > 10:
        state["path_taken"] = "high"
    else:
        state["path_taken"] = "low"
    return state


def process_high(state: ConditionalState) -> ConditionalState:
    """Process high values."""
    state["result"] = f"High value: {state['value']} * 2 = {state['value'] * 2}"
    return state


def process_low(state: ConditionalState) -> ConditionalState:
    """Process low values."""
    state["result"] = f"Low value: {state['value']} + 10 = {state['value'] + 10}"
    return state


def route_by_value(state: ConditionalState) -> Literal["high", "low"]:
    """Route based on path taken."""
    return state["path_taken"]


# Build the graph
workflow = StateGraph(ConditionalState)

# Add nodes
workflow.add_node("check", check_value)
workflow.add_node("high", process_high)
workflow.add_node("low", process_low)

# Set entry point
workflow.set_entry_point("check")

# Add conditional routing
workflow.add_conditional_edges(
    "check",
    route_by_value,
    {
        "high": "high",
        "low": "low"
    }
)

# Add terminal edges
workflow.add_edge("high", END)
workflow.add_edge("low", END)

# Compile
graph = workflow.compile()