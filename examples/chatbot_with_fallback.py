"""
Real-world example: Chatbot with Fallback Handling
Intelligent conversation routing with graceful degradation.
"""

from typing import TypedDict, Literal, List
from langgraph.graph import StateGraph, END


class ChatState(TypedDict):
    """State for chatbot workflow."""
    user_message: str
    conversation_history: List[str]
    intent: str  # greeting, question, command, unknown
    confidence: float
    response: str
    context: str
    fallback_triggered: bool
    escalate_to_human: bool


def detect_intent(state: ChatState) -> ChatState:
    """Detect user intent from message."""
    message = state["user_message"].lower()

    # Simple intent detection
    if any(word in message for word in ["hello", "hi", "hey"]):
        state["intent"] = "greeting"
        state["confidence"] = 0.9
    elif any(word in message for word in ["what", "how", "why", "when", "where", "who"]):
        state["intent"] = "question"
        state["confidence"] = 0.8
    elif any(word in message for word in ["please", "can you", "could you"]):
        state["intent"] = "command"
        state["confidence"] = 0.7
    else:
        state["intent"] = "unknown"
        state["confidence"] = 0.3

    return state


def handle_greeting(state: ChatState) -> ChatState:
    """Handle greeting intent."""
    greetings = [
        "Hello! How can I help you today?",
        "Hi there! What can I do for you?",
        "Hey! I'm here to assist you.",
    ]

    # Simple contextual response
    history_len = len(state["conversation_history"])
    state["response"] = greetings[history_len % len(greetings)]
    return state


def handle_question(state: ChatState) -> ChatState:
    """Handle question intent using knowledge base."""
    message = state["user_message"].lower()

    # Simplified knowledge base lookup
    if "hours" in message or "open" in message:
        state["response"] = "We're open Monday-Friday, 9 AM to 5 PM."
        state["context"] = "hours"
    elif "price" in message or "cost" in message:
        state["response"] = "Our pricing starts at $10/month. Would you like more details?"
        state["context"] = "pricing"
    elif "support" in message or "help" in message:
        state["response"] = "I'm here to help! You can also email support@example.com."
        state["context"] = "support"
    else:
        # Question not recognized - low confidence
        state["confidence"] = 0.4
        state["response"] = ""

    return state


def handle_command(state: ChatState) -> ChatState:
    """Handle command/request intent."""
    message = state["user_message"].lower()

    if "cancel" in message or "refund" in message:
        state["response"] = "I can help with that. Let me transfer you to our billing team."
        state["escalate_to_human"] = True
    elif "update" in message and "email" in message:
        state["response"] = "To update your email, please go to Settings > Account."
        state["context"] = "account_update"
    elif "reset" in message and "password" in message:
        state["response"] = "I'll send you a password reset link. Check your email in a moment."
        state["context"] = "password_reset"
    else:
        state["confidence"] = 0.4
        state["response"] = ""

    return state


def fallback_response(state: ChatState) -> ChatState:
    """Provide fallback response when confidence is low."""
    state["fallback_triggered"] = True

    fallback_responses = [
        "I'm not sure I understand. Could you rephrase that?",
        "I didn't quite get that. Can you provide more details?",
        "Hmm, I'm having trouble with that request. Can you try asking differently?",
    ]

    history_len = len(state["conversation_history"])
    state["response"] = fallback_responses[history_len % len(fallback_responses)]

    # After multiple fallbacks, escalate
    if state["conversation_history"].count("fallback") >= 2:
        state["escalate_to_human"] = True
        state["response"] += " Would you like to speak with a human agent?"

    return state


def route_after_intent(state: ChatState) -> Literal["greeting", "question", "command", "fallback"]:
    """Route based on detected intent."""
    if state["confidence"] < 0.5:
        return "fallback"
    else:
        return state["intent"]


def route_after_response(state: ChatState) -> Literal["fallback", END]:
    """Check if response was generated successfully."""
    if state["response"] == "" and state["confidence"] < 0.6:
        return "fallback"
    else:
        return END


# Build the graph
workflow = StateGraph(ChatState)

# Add nodes
workflow.add_node("detect_intent", detect_intent)
workflow.add_node("greeting", handle_greeting)
workflow.add_node("question", handle_question)
workflow.add_node("command", handle_command)
workflow.add_node("fallback", fallback_response)

# Add edges
workflow.set_entry_point("detect_intent")

workflow.add_conditional_edges(
    "detect_intent",
    route_after_intent,
    {
        "greeting": "greeting",
        "question": "question",
        "command": "command",
        "fallback": "fallback",
    }
)

workflow.add_edge("greeting", END)
workflow.add_edge("fallback", END)

workflow.add_conditional_edges(
    "question",
    route_after_response,
    {
        "fallback": "fallback",
        END: END,
    }
)

workflow.add_conditional_edges(
    "command",
    route_after_response,
    {
        "fallback": "fallback",
        END: END,
    }
)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_messages = [
        "Hello!",
        "What are your business hours?",
        "Can you help me reset my password?",
        "Tell me about quantum physics in detail",  # Should trigger fallback
    ]

    for msg in test_messages:
        state = {
            "user_message": msg,
            "conversation_history": [],
            "intent": "",
            "confidence": 0.0,
            "response": "",
            "context": "",
            "fallback_triggered": False,
            "escalate_to_human": False,
        }

        result = app.invoke(state)
        print(f"\nUser: {msg}")
        print(f"Intent: {result['intent']} (confidence: {result['confidence']:.2f})")
        print(f"Bot: {result['response']}")
        if result['fallback_triggered']:
            print("(Fallback triggered)")
        if result['escalate_to_human']:
            print("(Escalated to human)")
