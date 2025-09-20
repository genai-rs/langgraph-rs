"""Linear workflow test fixture - simple sequential execution."""

from typing import TypedDict
from langgraph.graph import StateGraph, END


class LinearState(TypedDict):
    """State for linear workflow."""
    input: str
    step1_output: str
    step2_output: str
    final_output: str


def step_one(state: LinearState) -> LinearState:
    """First processing step."""
    state["step1_output"] = f"Processed: {state['input']}"
    return state


def step_two(state: LinearState) -> LinearState:
    """Second processing step."""
    state["step2_output"] = f"Enhanced: {state['step1_output']}"
    return state


def finalize(state: LinearState) -> LinearState:
    """Final step."""
    state["final_output"] = f"Complete: {state['step2_output']}"
    return state


# Build the graph
workflow = StateGraph(LinearState)

# Add nodes
workflow.add_node("step1", step_one)
workflow.add_node("step2", step_two)
workflow.add_node("finalize", finalize)

# Add edges (linear flow)
workflow.set_entry_point("step1")
workflow.add_edge("step1", "step2")
workflow.add_edge("step2", "finalize")
workflow.add_edge("finalize", END)

# Compile
graph = workflow.compile()