"""
Real-world example: API Request Router with Retry Logic
Intelligent API routing with circuit breaker and fallback strategies.
"""

from typing import TypedDict, Literal, List
from langgraph.graph import StateGraph, END


class APIState(TypedDict):
    """State for API routing workflow."""
    request_path: str
    request_method: str
    request_body: str
    target_service: str
    retry_count: int
    max_retries: int
    response_status: int
    response_body: str
    error_message: str
    use_cache: bool
    use_fallback: bool
    circuit_open: bool


def route_request(state: APIState) -> APIState:
    """Route incoming request to appropriate service."""
    path = state["request_path"]

    if path.startswith("/api/users"):
        state["target_service"] = "user_service"
    elif path.startswith("/api/products"):
        state["target_service"] = "product_service"
    elif path.startswith("/api/orders"):
        state["target_service"] = "order_service"
    else:
        state["target_service"] = "unknown"

    return state


def check_cache(state: APIState) -> APIState:
    """Check if request can be served from cache."""
    # Only cache GET requests
    if state["request_method"] == "GET":
        # Simulate cache lookup
        cache_key = f"{state['target_service']}:{state['request_path']}"

        # Simple cache hit simulation (in real system, would use Redis, etc.)
        if "list" in state["request_path"] or "get" in state["request_path"]:
            state["use_cache"] = True
            state["response_status"] = 200
            state["response_body"] = f"{{\"cached\": true, \"service\": \"{state['target_service']}\"}}"
        else:
            state["use_cache"] = False
    else:
        state["use_cache"] = False

    return state


def call_primary_service(state: APIState) -> APIState:
    """Call the primary target service."""
    # Simulate API call
    service = state["target_service"]

    # Simulate occasional failures
    if state["retry_count"] > 0:
        # First retry usually succeeds
        state["response_status"] = 200
        state["response_body"] = f"{{\"success\": true, \"service\": \"{service}\"}}"
    elif "unknown" in service:
        # Unknown service
        state["response_status"] = 404
        state["error_message"] = "Service not found"
    else:
        # Simulate 10% failure rate on first attempt
        import random
        if random.random() > 0.9:
            state["response_status"] = 500
            state["error_message"] = "Service temporarily unavailable"
        else:
            state["response_status"] = 200
            state["response_body"] = f"{{\"success\": true, \"service\": \"{service}\"}}"

    return state


def check_circuit_breaker(state: APIState) -> APIState:
    """Check if circuit breaker should open."""
    # In real system, would track failure rates over time
    if state["retry_count"] >= state["max_retries"]:
        state["circuit_open"] = True

    return state


def retry_request(state: APIState) -> APIState:
    """Retry failed request with backoff."""
    state["retry_count"] += 1

    # In real system, would implement exponential backoff
    # For now, just increment retry counter

    return state


def use_fallback_service(state: APIState) -> APIState:
    """Use fallback/degraded service."""
    state["use_fallback"] = True
    state["response_status"] = 200
    state["response_body"] = f"{{\"degraded\": true, \"message\": \"Using cached/fallback data\"}}"

    return state


def format_error_response(state: APIState) -> APIState:
    """Format error response for client."""
    state["response_status"] = 503
    state["response_body"] = f"{{\"error\": \"{state['error_message']}\", \"retry_after\": 60}}"

    return state


def route_after_cache(state: APIState) -> Literal["primary", END]:
    """Skip API call if cache hit."""
    if state["use_cache"]:
        return END
    else:
        return "primary"


def route_after_primary(state: APIState) -> Literal["check_circuit", END]:
    """Check if request succeeded."""
    if state["response_status"] == 200:
        return END
    else:
        return "check_circuit"


def route_after_circuit_check(state: APIState) -> Literal["retry", "fallback", "error"]:
    """Decide retry strategy based on circuit breaker state."""
    if state["circuit_open"]:
        return "error"
    elif state["retry_count"] < state["max_retries"]:
        return "retry"
    else:
        return "fallback"


# Build the graph
workflow = StateGraph(APIState)

# Add nodes
workflow.add_node("route", route_request)
workflow.add_node("cache", check_cache)
workflow.add_node("primary", call_primary_service)
workflow.add_node("check_circuit", check_circuit_breaker)
workflow.add_node("retry", retry_request)
workflow.add_node("fallback", use_fallback_service)
workflow.add_node("error", format_error_response)

# Add edges
workflow.set_entry_point("route")
workflow.add_edge("route", "cache")

workflow.add_conditional_edges(
    "cache",
    route_after_cache,
    {
        "primary": "primary",
        END: END,
    }
)

workflow.add_conditional_edges(
    "primary",
    route_after_primary,
    {
        "check_circuit": "check_circuit",
        END: END,
    }
)

workflow.add_conditional_edges(
    "check_circuit",
    route_after_circuit_check,
    {
        "retry": "retry",
        "fallback": "fallback",
        "error": "error",
    }
)

workflow.add_edge("retry", "primary")  # Loop back
workflow.add_edge("fallback", END)
workflow.add_edge("error", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_requests = [
        {
            "request_path": "/api/users/list",
            "request_method": "GET",
            "request_body": "",
            "target_service": "",
            "retry_count": 0,
            "max_retries": 3,
            "response_status": 0,
            "response_body": "",
            "error_message": "",
            "use_cache": False,
            "use_fallback": False,
            "circuit_open": False,
        },
        {
            "request_path": "/api/products/123",
            "request_method": "POST",
            "request_body": '{"name": "Widget"}',
            "target_service": "",
            "retry_count": 0,
            "max_retries": 3,
            "response_status": 0,
            "response_body": "",
            "error_message": "",
            "use_cache": False,
            "use_fallback": False,
            "circuit_open": False,
        },
    ]

    for req in test_requests:
        result = app.invoke(req)
        print(f"\nRequest: {req['request_method']} {req['request_path']}")
        print(f"Service: {result['target_service']}")
        print(f"Status: {result['response_status']}")
        print(f"Cached: {result['use_cache']}")
        print(f"Retries: {result['retry_count']}")
        print(f"Response: {result['response_body']}")
