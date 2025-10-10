"""
Real-world example: Data Validation and Enrichment Pipeline
Multi-stage data processing with validation, enrichment, and quality checks.
"""

from typing import TypedDict, Literal, List, Dict
from langgraph.graph import StateGraph, END


class DataState(TypedDict):
    """State for data processing workflow."""
    raw_data: Dict[str, str]
    data_type: str
    validation_errors: List[str]
    enriched_data: Dict[str, str]
    quality_score: float
    needs_manual_review: bool
    processed: bool
    timestamp: str


def classify_data(state: DataState) -> DataState:
    """Classify incoming data type."""
    data = state["raw_data"]

    if "email" in data and "name" in data:
        state["data_type"] = "contact"
    elif "product_id" in data and "price" in data:
        state["data_type"] = "product"
    elif "user_id" in data and "action" in data:
        state["data_type"] = "event"
    else:
        state["data_type"] = "unknown"

    return state


def validate_schema(state: DataState) -> DataState:
    """Validate data against schema."""
    errors = []
    data = state["raw_data"]
    data_type = state["data_type"]

    # Type-specific validation
    if data_type == "contact":
        if "email" not in data:
            errors.append("Missing required field: email")
        elif "@" not in data.get("email", ""):
            errors.append("Invalid email format")

        if "name" not in data:
            errors.append("Missing required field: name")
        elif len(data.get("name", "")) < 2:
            errors.append("Name too short")

    elif data_type == "product":
        if "product_id" not in data:
            errors.append("Missing required field: product_id")

        if "price" not in data:
            errors.append("Missing required field: price")
        else:
            try:
                price = float(data["price"])
                if price < 0:
                    errors.append("Price cannot be negative")
            except ValueError:
                errors.append("Price must be a number")

    elif data_type == "event":
        if "user_id" not in data:
            errors.append("Missing required field: user_id")
        if "action" not in data:
            errors.append("Missing required field: action")

    state["validation_errors"] = errors
    return state


def enrich_data(state: DataState) -> DataState:
    """Enrich data with additional information."""
    enriched = state["raw_data"].copy()
    data_type = state["data_type"]

    # Type-specific enrichment
    if data_type == "contact":
        # Add email domain
        if "email" in enriched:
            domain = enriched["email"].split("@")[-1]
            enriched["email_domain"] = domain

            # Classify email provider
            if domain in ["gmail.com", "yahoo.com", "hotmail.com"]:
                enriched["email_provider"] = "consumer"
            else:
                enriched["email_provider"] = "business"

        # Normalize name
        if "name" in enriched:
            enriched["name"] = enriched["name"].title()

    elif data_type == "product":
        # Add price tier
        if "price" in enriched:
            price = float(enriched["price"])
            if price < 10:
                enriched["price_tier"] = "budget"
            elif price < 100:
                enriched["price_tier"] = "standard"
            else:
                enriched["price_tier"] = "premium"

    elif data_type == "event":
        # Add timestamp if missing
        import datetime
        if "timestamp" not in enriched:
            enriched["timestamp"] = datetime.datetime.now().isoformat()

    state["enriched_data"] = enriched
    return state


def calculate_quality_score(state: DataState) -> DataState:
    """Calculate data quality score."""
    score = 1.0

    # Deduct points for validation errors
    error_count = len(state["validation_errors"])
    score -= (error_count * 0.2)

    # Bonus for enriched fields
    enriched_count = len(state["enriched_data"]) - len(state["raw_data"])
    score += (enriched_count * 0.1)

    # Ensure score is between 0 and 1
    score = max(0.0, min(1.0, score))

    state["quality_score"] = score

    # Flag for manual review if quality is low
    if score < 0.6:
        state["needs_manual_review"] = True

    return state


def manual_review_queue(state: DataState) -> DataState:
    """Queue data for manual review."""
    # In real system, would add to review queue
    state["processed"] = False  # Not fully processed
    return state


def finalize_processing(state: DataState) -> DataState:
    """Finalize data processing."""
    import datetime
    state["processed"] = True
    state["timestamp"] = datetime.datetime.now().isoformat()
    return state


def route_after_validation(state: DataState) -> Literal["enrich", END]:
    """Decide if data should be enriched or rejected."""
    # If too many errors, skip enrichment
    if len(state["validation_errors"]) > 3:
        return END
    else:
        return "enrich"


def route_after_quality(state: DataState) -> Literal["manual_review", "finalize"]:
    """Route based on quality score."""
    if state["needs_manual_review"]:
        return "manual_review"
    else:
        return "finalize"


# Build the graph
workflow = StateGraph(DataState)

# Add nodes
workflow.add_node("classify", classify_data)
workflow.add_node("validate", validate_schema)
workflow.add_node("enrich", enrich_data)
workflow.add_node("quality", calculate_quality_score)
workflow.add_node("manual_review", manual_review_queue)
workflow.add_node("finalize", finalize_processing)

# Add edges
workflow.set_entry_point("classify")
workflow.add_edge("classify", "validate")

workflow.add_conditional_edges(
    "validate",
    route_after_validation,
    {
        "enrich": "enrich",
        END: END,
    }
)

workflow.add_edge("enrich", "quality")

workflow.add_conditional_edges(
    "quality",
    route_after_quality,
    {
        "manual_review": "manual_review",
        "finalize": "finalize",
    }
)

workflow.add_edge("manual_review", END)
workflow.add_edge("finalize", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_data = [
        {
            "raw_data": {"name": "john doe", "email": "john@example.com"},
            "data_type": "",
            "validation_errors": [],
            "enriched_data": {},
            "quality_score": 0.0,
            "needs_manual_review": False,
            "processed": False,
            "timestamp": "",
        },
        {
            "raw_data": {"product_id": "P123", "price": "99.99"},
            "data_type": "",
            "validation_errors": [],
            "enriched_data": {},
            "quality_score": 0.0,
            "needs_manual_review": False,
            "processed": False,
            "timestamp": "",
        },
        {
            "raw_data": {"email": "invalid-email"},  # Missing name, invalid email
            "data_type": "",
            "validation_errors": [],
            "enriched_data": {},
            "quality_score": 0.0,
            "needs_manual_review": False,
            "processed": False,
            "timestamp": "",
        },
    ]

    for data in test_data:
        result = app.invoke(data)
        print(f"\nRaw data: {data['raw_data']}")
        print(f"Type: {result['data_type']}")
        print(f"Errors: {result['validation_errors']}")
        print(f"Quality: {result['quality_score']:.2f}")
        print(f"Enriched: {result['enriched_data']}")
        print(f"Processed: {result['processed']}")
