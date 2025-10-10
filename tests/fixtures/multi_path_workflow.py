"""Workflow with multiple independent paths that merge."""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class MultiPathState(TypedDict):
    """State for multi-path workflow."""
    input_value: int
    path_a_result: int
    path_b_result: int
    path_c_result: int
    final_result: int
    chosen_path: str


def start(state: MultiPathState) -> MultiPathState:
    """Entry point - determines which path to take."""
    state["chosen_path"] = "unknown"
    return state


def router(state: MultiPathState) -> Literal["path_a", "path_b", "path_c"]:
    """Route to different paths based on input."""
    value = state["input_value"]

    if value < 33:
        return "path_a"
    elif value < 66:
        return "path_b"
    else:
        return "path_c"


def path_a(state: MultiPathState) -> MultiPathState:
    """Processing path A."""
    state["path_a_result"] = state["input_value"] * 2
    state["chosen_path"] = "A"
    return state


def path_b(state: MultiPathState) -> MultiPathState:
    """Processing path B."""
    state["path_b_result"] = state["input_value"] + 100
    state["chosen_path"] = "B"
    return state


def path_c(state: MultiPathState) -> MultiPathState:
    """Processing path C."""
    state["path_c_result"] = state["input_value"] ** 2
    state["chosen_path"] = "C"
    return state


def merge(state: MultiPathState) -> MultiPathState:
    """Merge results from all paths."""
    # Sum all non-zero results
    total = state["path_a_result"] + state["path_b_result"] + state["path_c_result"]
    state["final_result"] = total
    return state


# Build the graph
workflow = StateGraph(MultiPathState)

# Add nodes
workflow.add_node("start", start)
workflow.add_node("path_a", path_a)
workflow.add_node("path_b", path_b)
workflow.add_node("path_c", path_c)
workflow.add_node("merge", merge)

# Set entry point
workflow.set_entry_point("start")

# Add conditional routing from start
workflow.add_conditional_edges(
    "start",
    router,
    {
        "path_a": "path_a",
        "path_b": "path_b",
        "path_c": "path_c"
    }
)

# All paths converge to merge
workflow.add_edge("path_a", "merge")
workflow.add_edge("path_b", "merge")
workflow.add_edge("path_c", "merge")
workflow.add_edge("merge", END)

# Compile the graph
graph = workflow.compile()


if __name__ == "__main__":
    for test_value in [10, 50, 80]:
        initial_state = {
            "input_value": test_value,
            "path_a_result": 0,
            "path_b_result": 0,
            "path_c_result": 0,
            "final_result": 0,
            "chosen_path": ""
        }

        result = graph.invoke(initial_state)
        print(f"\nInput: {test_value}")
        print(f"Chosen path: {result['chosen_path']}")
        print(f"Final result: {result['final_result']}")
