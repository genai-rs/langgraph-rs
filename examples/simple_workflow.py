"""Example LangGraph workflow for testing conversion."""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class AgentState(TypedDict):
    """State schema for the agent."""
    messages: list[str]
    context: dict
    next_action: str
    counter: int


def start_node(state: AgentState) -> AgentState:
    """Initial processing node."""
    state["messages"].append("Starting workflow")
    state["counter"] = 0
    state["next_action"] = "process"
    return state


def process_node(state: AgentState) -> AgentState:
    """Main processing logic."""
    state["counter"] += 1
    state["messages"].append(f"Processing step {state['counter']}")

    if state["counter"] >= 3:
        state["next_action"] = "end"
    else:
        state["next_action"] = "process"

    return state


def end_node(state: AgentState) -> AgentState:
    """Cleanup and finalization."""
    state["messages"].append("Workflow complete")
    return state


def route_next(state: AgentState) -> Literal["process", "end"]:
    """Determine next node based on state."""
    return state.get("next_action", "end")


# Build the graph
workflow = StateGraph(AgentState)

# Add nodes
workflow.add_node("start", start_node)
workflow.add_node("process", process_node)
workflow.add_node("end", end_node)

# Set entry point
workflow.set_entry_point("start")

# Add edges
workflow.add_edge("start", "process")
workflow.add_conditional_edges(
    "process",
    route_next,
    {
        "process": "process",  # Loop back
        "end": "end"
    }
)
workflow.add_edge("end", END)

# Compile the graph
graph = workflow.compile()

if __name__ == "__main__":
    # Test the graph
    initial_state = {
        "messages": [],
        "context": {},
        "next_action": "",
        "counter": 0
    }

    result = graph.invoke(initial_state)
    print("Final state:", result)