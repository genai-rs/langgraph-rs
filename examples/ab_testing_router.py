"""
Real-world example: A/B Testing Router
Dynamic experiment routing with statistical tracking and feature flags.
"""

from typing import TypedDict, Literal, List, Dict
from langgraph.graph import StateGraph, END


class ABTestState(TypedDict):
    """State for A/B testing workflow."""
    user_id: str
    experiment_id: str
    variant: str  # control, variant_a, variant_b, etc.
    user_segment: str
    feature_flags: Dict[str, bool]
    experiment_config: Dict[str, any]
    tracking_data: Dict[str, any]
    conversion_tracked: bool
    override_variant: str  # For manual overrides


def identify_user(state: ABTestState) -> ABTestState:
    """Identify user and determine segment."""
    user_id = state["user_id"]

    # Simulate user segmentation
    # In real system, would query user database/profile

    # Simple hash-based segmentation for consistency
    user_hash = hash(user_id) % 100

    if user_hash < 20:
        state["user_segment"] = "new_users"
    elif user_hash < 50:
        state["user_segment"] = "active_users"
    else:
        state["user_segment"] = "power_users"

    return state


def load_experiment_config(state: ABTestState) -> ABTestState:
    """Load experiment configuration and targeting rules."""
    experiment_id = state["experiment_id"]

    # Simulate experiment config lookup
    # In real system, would query experiment service/database

    experiment_configs = {
        "homepage_redesign": {
            "variants": ["control", "variant_a", "variant_b"],
            "traffic_allocation": {"control": 0.4, "variant_a": 0.3, "variant_b": 0.3},
            "target_segments": ["new_users", "active_users"],
            "feature_flags": {"new_homepage": True},
        },
        "pricing_test": {
            "variants": ["control", "variant_a"],
            "traffic_allocation": {"control": 0.5, "variant_a": 0.5},
            "target_segments": ["power_users"],
            "feature_flags": {"new_pricing": True},
        },
        "checkout_flow": {
            "variants": ["control", "variant_a"],
            "traffic_allocation": {"control": 0.6, "variant_a": 0.4},
            "target_segments": ["new_users", "active_users", "power_users"],
            "feature_flags": {"streamlined_checkout": True},
        },
    }

    config = experiment_configs.get(experiment_id, {})
    state["experiment_config"] = config

    # Set feature flags
    state["feature_flags"] = config.get("feature_flags", {})

    return state


def check_eligibility(state: ABTestState) -> ABTestState:
    """Check if user is eligible for experiment."""
    segment = state["user_segment"]
    config = state["experiment_config"]

    target_segments = config.get("target_segments", [])

    # User is eligible if in target segment
    eligible = segment in target_segments

    if not eligible:
        # Default to control for ineligible users
        state["variant"] = "control"
        state["tracking_data"]["eligibility"] = "excluded"
    else:
        state["tracking_data"]["eligibility"] = "included"

    return state


def assign_variant(state: ABTestState) -> ABTestState:
    """Assign user to experiment variant."""
    # Check for manual override
    if state["override_variant"]:
        state["variant"] = state["override_variant"]
        state["tracking_data"]["assignment"] = "override"
        return state

    config = state["experiment_config"]
    variants = config.get("variants", ["control"])
    allocation = config.get("traffic_allocation", {})

    # Consistent hash-based assignment
    user_hash = hash(state["user_id"] + state["experiment_id"]) % 100

    # Allocate based on traffic split
    cumulative = 0
    for variant, percentage in allocation.items():
        cumulative += int(percentage * 100)
        if user_hash < cumulative:
            state["variant"] = variant
            break
    else:
        # Default to control
        state["variant"] = "control"

    state["tracking_data"]["assignment"] = "algorithmic"

    return state


def activate_features(state: ABTestState) -> ABTestState:
    """Activate variant-specific features."""
    variant = state["variant"]

    # Variant-specific feature flags
    variant_features = {
        "variant_a": {"use_new_design": True, "show_recommendations": True},
        "variant_b": {"use_new_design": True, "show_social_proof": True},
        "control": {"use_new_design": False},
    }

    features = variant_features.get(variant, {})

    # Merge with experiment feature flags
    state["feature_flags"].update(features)

    return state


def track_assignment(state: ABTestState) -> ABTestState:
    """Track variant assignment for analytics."""
    # Simulate analytics tracking
    # In real system, would send to analytics service (Mixpanel, Amplitude, etc.)

    tracking_event = {
        "event": "experiment_assignment",
        "experiment_id": state["experiment_id"],
        "variant": state["variant"],
        "user_id": state["user_id"],
        "user_segment": state["user_segment"],
        "timestamp": "2024-01-15T12:00:00Z",
    }

    state["tracking_data"]["assignment_event"] = tracking_event

    return state


def route_after_eligibility(state: ABTestState) -> Literal["assign", "track"]:
    """Route based on eligibility."""
    if state["tracking_data"].get("eligibility") == "excluded":
        # Skip assignment for ineligible users
        return "track"
    else:
        return "assign"


# Build the graph
workflow = StateGraph(ABTestState)

# Add nodes
workflow.add_node("identify", identify_user)
workflow.add_node("load_config", load_experiment_config)
workflow.add_node("check_eligibility", check_eligibility)
workflow.add_node("assign", assign_variant)
workflow.add_node("activate", activate_features)
workflow.add_node("track", track_assignment)

# Add edges
workflow.set_entry_point("identify")
workflow.add_edge("identify", "load_config")
workflow.add_edge("load_config", "check_eligibility")

workflow.add_conditional_edges(
    "check_eligibility",
    route_after_eligibility,
    {
        "assign": "assign",
        "track": "track",
    }
)

workflow.add_edge("assign", "activate")
workflow.add_edge("activate", "track")
workflow.add_edge("track", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_cases = [
        {
            "user_id": "user_123",
            "experiment_id": "homepage_redesign",
            "variant": "",
            "user_segment": "",
            "feature_flags": {},
            "experiment_config": {},
            "tracking_data": {},
            "conversion_tracked": False,
            "override_variant": "",
        },
        {
            "user_id": "user_456",
            "experiment_id": "pricing_test",
            "variant": "",
            "user_segment": "",
            "feature_flags": {},
            "experiment_config": {},
            "tracking_data": {},
            "conversion_tracked": False,
            "override_variant": "",
        },
        {
            "user_id": "user_789",
            "experiment_id": "checkout_flow",
            "variant": "",
            "user_segment": "",
            "feature_flags": {},
            "experiment_config": {},
            "tracking_data": {},
            "conversion_tracked": False,
            "override_variant": "variant_a",  # Manual override
        },
    ]

    for test in test_cases:
        result = app.invoke(test)
        print(f"\nUser: {test['user_id']}")
        print(f"Experiment: {result['experiment_id']}")
        print(f"Segment: {result['user_segment']}")
        print(f"Variant: {result['variant']}")
        print(f"Eligibility: {result['tracking_data'].get('eligibility')}")
        print(f"Assignment: {result['tracking_data'].get('assignment')}")
        print(f"Feature Flags: {result['feature_flags']}")
