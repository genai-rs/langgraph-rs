"""Workflow with complex Python types."""

from typing import TypedDict, Optional
from dataclasses import dataclass
from langgraph.graph import StateGraph, END


@dataclass
class UserInfo:
    """Custom dataclass for user information."""
    name: str
    age: int
    email: Optional[str] = None


class ComplexState(TypedDict):
    """State with complex nested types."""
    users: list[dict]
    metadata: dict[str, any]
    current_user: Optional[dict]
    scores: dict[str, float]
    flags: dict[str, bool]


def init_node(state: ComplexState) -> ComplexState:
    """Initialize with complex data."""
    state["metadata"]["initialized"] = True
    state["metadata"]["version"] = "1.0"
    return state


def process_users(state: ComplexState) -> ComplexState:
    """Process user list."""
    for user in state["users"]:
        user["processed"] = True
        state["scores"][user["name"]] = 0.0

    if state["users"]:
        state["current_user"] = state["users"][0]

    return state


def calculate_scores(state: ComplexState) -> ComplexState:
    """Calculate scores for users."""
    for user in state["users"]:
        name = user["name"]
        age = user.get("age", 0)
        state["scores"][name] = float(age * 1.5)

    state["flags"]["scores_calculated"] = True
    return state


def finalize(state: ComplexState) -> ComplexState:
    """Final processing."""
    state["metadata"]["total_users"] = len(state["users"])
    state["metadata"]["avg_score"] = sum(state["scores"].values()) / len(state["scores"]) if state["scores"] else 0.0
    state["flags"]["complete"] = True
    return state


# Build the graph
workflow = StateGraph(ComplexState)

# Add nodes
workflow.add_node("init", init_node)
workflow.add_node("process_users", process_users)
workflow.add_node("calculate_scores", calculate_scores)
workflow.add_node("finalize", finalize)

# Set entry point
workflow.set_entry_point("init")

# Add edges
workflow.add_edge("init", "process_users")
workflow.add_edge("process_users", "calculate_scores")
workflow.add_edge("calculate_scores", "finalize")
workflow.add_edge("finalize", END)

# Compile the graph
graph = workflow.compile()


if __name__ == "__main__":
    initial_state = {
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25},
            {"name": "Charlie", "age": 35}
        ],
        "metadata": {},
        "current_user": None,
        "scores": {},
        "flags": {"scores_calculated": False, "complete": False}
    }

    result = graph.invoke(initial_state)
    print(f"Processed {result['metadata']['total_users']} users")
    print(f"Average score: {result['metadata']['avg_score']}")
    print(f"Scores: {result['scores']}")
