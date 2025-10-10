"""
Real-world example: Customer Support Agent
Handles customer queries with routing to different support tiers.
"""

from typing import TypedDict, Literal
from langgraph.graph import StateGraph, END


class SupportState(TypedDict):
    """State for customer support workflow."""
    customer_message: str
    sentiment: str  # positive, neutral, negative
    category: str  # technical, billing, general
    priority: str  # low, medium, high
    response: str
    escalated: bool
    resolved: bool


def analyze_message(state: SupportState) -> SupportState:
    """Analyze customer message for sentiment and category."""
    # TODO: Real implementation would use NLP/LLM here
    message = state["customer_message"].lower()

    # Simple sentiment analysis
    if any(word in message for word in ["angry", "frustrated", "terrible"]):
        state["sentiment"] = "negative"
    elif any(word in message for word in ["great", "happy", "excellent"]):
        state["sentiment"] = "positive"
    else:
        state["sentiment"] = "neutral"

    # Simple categorization
    if any(word in message for word in ["bug", "error", "crash", "technical"]):
        state["category"] = "technical"
    elif any(word in message for word in ["payment", "billing", "charge", "refund"]):
        state["category"] = "billing"
    else:
        state["category"] = "general"

    # Set priority based on sentiment and category
    if state["sentiment"] == "negative" and state["category"] in ["technical", "billing"]:
        state["priority"] = "high"
    elif state["sentiment"] == "negative" or state["category"] == "billing":
        state["priority"] = "medium"
    else:
        state["priority"] = "low"

    return state


def handle_tier1(state: SupportState) -> SupportState:
    """Tier 1 support - handles simple queries."""
    if state["category"] == "general" and state["priority"] == "low":
        state["response"] = f"Thank you for contacting us about: {state['customer_message'][:50]}..."
        state["resolved"] = True
    else:
        state["resolved"] = False
    return state


def handle_tier2(state: SupportState) -> SupportState:
    """Tier 2 support - handles technical and billing issues."""
    if state["category"] in ["technical", "billing"] and state["priority"] in ["low", "medium"]:
        state["response"] = f"Our specialist will handle your {state['category']} issue."
        state["resolved"] = True
    else:
        state["resolved"] = False
    return state


def escalate(state: SupportState) -> SupportState:
    """Escalate to senior support."""
    state["response"] = "This has been escalated to our senior support team."
    state["escalated"] = True
    state["resolved"] = True
    return state


def route_after_analysis(state: SupportState) -> Literal["tier1", "tier2", "escalate"]:
    """Route to appropriate support tier based on analysis."""
    if state["priority"] == "high":
        return "escalate"
    elif state["category"] in ["technical", "billing"]:
        return "tier2"
    else:
        return "tier1"


def route_after_tier1(state: SupportState) -> Literal["tier2", END]:
    """Route from tier1 based on resolution."""
    if state["resolved"]:
        return END
    else:
        return "tier2"


def route_after_tier2(state: SupportState) -> Literal["escalate", END]:
    """Route from tier2 based on resolution."""
    if state["resolved"]:
        return END
    else:
        return "escalate"


# Build the graph
workflow = StateGraph(SupportState)

# Add nodes
workflow.add_node("analyze", analyze_message)
workflow.add_node("tier1", handle_tier1)
workflow.add_node("tier2", handle_tier2)
workflow.add_node("escalate", escalate)

# Add edges
workflow.set_entry_point("analyze")

workflow.add_conditional_edges(
    "analyze",
    route_after_analysis,
    {
        "tier1": "tier1",
        "tier2": "tier2",
        "escalate": "escalate",
    }
)

workflow.add_conditional_edges(
    "tier1",
    route_after_tier1,
    {
        "tier2": "tier2",
        END: END,
    }
)

workflow.add_conditional_edges(
    "tier2",
    route_after_tier2,
    {
        "escalate": "escalate",
        END: END,
    }
)

workflow.add_edge("escalate", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_cases = [
        {"customer_message": "I love your product!", "sentiment": "", "category": "", "priority": "", "response": "", "escalated": False, "resolved": False},
        {"customer_message": "I found a critical bug in the app", "sentiment": "", "category": "", "priority": "", "response": "", "escalated": False, "resolved": False},
        {"customer_message": "I was charged twice for my subscription", "sentiment": "", "category": "", "priority": "", "response": "", "escalated": False, "resolved": False},
    ]

    for test in test_cases:
        result = app.invoke(test)
        print(f"\nInput: {test['customer_message']}")
        print(f"Category: {result['category']}, Priority: {result['priority']}")
        print(f"Response: {result['response']}")
        print(f"Escalated: {result['escalated']}")
