"""
Real-world example: Content Moderation System
Multi-stage content filtering with escalation for edge cases.
"""

from typing import TypedDict, Literal, List
from langgraph.graph import StateGraph, END


class ModerationState(TypedDict):
    """State for content moderation workflow."""
    content: str
    content_type: str  # text, image, video
    flags: List[str]
    severity: str  # safe, warning, dangerous
    auto_decision: str  # approve, reject, review
    human_review_required: bool
    final_decision: str
    reason: str


def keyword_filter(state: ModerationState) -> ModerationState:
    """Check for prohibited keywords."""
    content = state["content"].lower()
    flags = state["flags"]

    prohibited_keywords = ["spam", "scam", "hack", "illegal"]

    for keyword in prohibited_keywords:
        if keyword in content:
            flags.append(f"keyword:{keyword}")

    state["flags"] = flags
    return state


def sentiment_check(state: ModerationState) -> ModerationState:
    """Analyze content sentiment for toxicity."""
    content = state["content"].lower()
    flags = state["flags"]

    toxic_patterns = ["hate", "violence", "threat", "abuse"]

    for pattern in toxic_patterns:
        if pattern in content:
            flags.append(f"toxic:{pattern}")

    state["flags"] = flags
    return state


def ml_classifier(state: ModerationState) -> ModerationState:
    """ML-based content classification."""
    # Simplified ML classification
    flag_count = len(state["flags"])

    if flag_count == 0:
        state["severity"] = "safe"
    elif flag_count <= 2:
        state["severity"] = "warning"
    else:
        state["severity"] = "dangerous"

    return state


def auto_moderator(state: ModerationState) -> ModerationState:
    """Make automatic moderation decision."""
    if state["severity"] == "safe":
        state["auto_decision"] = "approve"
        state["final_decision"] = "approve"
        state["reason"] = "No issues detected"
    elif state["severity"] == "dangerous":
        state["auto_decision"] = "reject"
        state["final_decision"] = "reject"
        state["reason"] = f"Dangerous content detected: {', '.join(state['flags'])}"
    else:
        state["auto_decision"] = "review"
        state["human_review_required"] = True

    return state


def human_review(state: ModerationState) -> ModerationState:
    """Simulate human review for edge cases."""
    # In real system, this would queue for human moderator
    state["final_decision"] = "review_pending"
    state["reason"] = "Flagged for human review: " + ", ".join(state["flags"])
    return state


def route_after_auto_moderator(state: ModerationState) -> Literal["human_review", END]:
    """Route based on auto-moderation decision."""
    if state["human_review_required"]:
        return "human_review"
    else:
        return END


# Build the graph
workflow = StateGraph(ModerationState)

# Add nodes
workflow.add_node("keyword_filter", keyword_filter)
workflow.add_node("sentiment_check", sentiment_check)
workflow.add_node("ml_classifier", ml_classifier)
workflow.add_node("auto_moderator", auto_moderator)
workflow.add_node("human_review", human_review)

# Add edges - linear pipeline with final conditional
workflow.set_entry_point("keyword_filter")
workflow.add_edge("keyword_filter", "sentiment_check")
workflow.add_edge("sentiment_check", "ml_classifier")
workflow.add_edge("ml_classifier", "auto_moderator")

workflow.add_conditional_edges(
    "auto_moderator",
    route_after_auto_moderator,
    {
        "human_review": "human_review",
        END: END,
    }
)

workflow.add_edge("human_review", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_cases = [
        {
            "content": "This is a normal message",
            "content_type": "text",
            "flags": [],
            "severity": "",
            "auto_decision": "",
            "human_review_required": False,
            "final_decision": "",
            "reason": "",
        },
        {
            "content": "Check out this spam offer",
            "content_type": "text",
            "flags": [],
            "severity": "",
            "auto_decision": "",
            "human_review_required": False,
            "final_decision": "",
            "reason": "",
        },
        {
            "content": "This is hate speech with violence and illegal threats",
            "content_type": "text",
            "flags": [],
            "severity": "",
            "auto_decision": "",
            "human_review_required": False,
            "final_decision": "",
            "reason": "",
        },
    ]

    for test in test_cases:
        result = app.invoke(test)
        print(f"\nContent: {test['content']}")
        print(f"Flags: {result['flags']}")
        print(f"Severity: {result['severity']}")
        print(f"Decision: {result['final_decision']}")
        print(f"Reason: {result['reason']}")
