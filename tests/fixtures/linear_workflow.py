"""Simple linear workflow - no branching, just sequential nodes."""

from typing import TypedDict
from langgraph.graph import StateGraph, END


class SimpleState(TypedDict):
    """Simple state with just a counter and messages."""
    counter: int
    messages: list[str]


def node_a(state: SimpleState) -> SimpleState:
    """First node in the chain."""
    state["counter"] += 1
    state["messages"].append("Node A executed")
    return state


def node_b(state: SimpleState) -> SimpleState:
    """Second node in the chain."""
    state["counter"] += 1
    state["messages"].append("Node B executed")
    return state


def node_c(state: SimpleState) -> SimpleState:
    """Final node in the chain."""
    state["counter"] += 1
    state["messages"].append("Node C executed")
    return state


# Build the graph
workflow = StateGraph(SimpleState)

# Add nodes
workflow.add_node("a", node_a)
workflow.add_node("b", node_b)
workflow.add_node("c", node_c)

# Set entry point
workflow.set_entry_point("a")

# Add edges: a -> b -> c -> END
workflow.add_edge("a", "b")
workflow.add_edge("b", "c")
workflow.add_edge("c", END)

# Compile the graph
graph = workflow.compile()


if __name__ == "__main__":
    initial_state = {"counter": 0, "messages": []}
    result = graph.invoke(initial_state)
    print(f"Final counter: {result['counter']}")
    print(f"Messages: {result['messages']}")
