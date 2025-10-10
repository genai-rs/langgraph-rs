"""
Real-world example: ETL (Extract, Transform, Load) Pipeline
Data pipeline with validation, transformation, and error handling.
"""

from typing import TypedDict, Literal, List, Dict
from langgraph.graph import StateGraph, END


class ETLState(TypedDict):
    """State for ETL pipeline workflow."""
    source_type: str  # database, api, file, stream
    source_config: Dict[str, str]
    raw_records: List[Dict[str, any]]
    transformed_records: List[Dict[str, any]]
    validation_errors: List[Dict[str, str]]
    record_count: int
    success_count: int
    error_count: int
    transformation_rules: List[str]
    destination_type: str  # database, warehouse, file, api
    load_status: str
    pipeline_stats: Dict[str, any]


def extract_data(state: ETLState) -> ETLState:
    """Extract data from source."""
    source_type = state["source_type"]
    raw_records = []

    # Simulate data extraction based on source type
    if source_type == "database":
        # Simulate database query
        raw_records = [
            {"id": "1", "name": "John Doe", "age": "30", "email": "john@example.com"},
            {"id": "2", "name": "Jane Smith", "age": "25", "email": "jane@example.com"},
            {"id": "3", "name": "Bob Wilson", "age": "invalid", "email": "bob@example"},  # Invalid age
        ]
    elif source_type == "api":
        # Simulate API call
        raw_records = [
            {"user_id": "101", "transaction_amount": "150.50", "date": "2024-01-15"},
            {"user_id": "102", "transaction_amount": "75.25", "date": "2024-01-16"},
        ]
    elif source_type == "file":
        # Simulate file reading
        raw_records = [
            {"product_id": "P1", "quantity": "100", "price": "29.99"},
            {"product_id": "P2", "quantity": "50", "price": "49.99"},
        ]

    state["raw_records"] = raw_records
    state["record_count"] = len(raw_records)

    return state


def validate_records(state: ETLState) -> ETLState:
    """Validate extracted records."""
    errors = []

    for i, record in enumerate(state["raw_records"]):
        # Check for required fields
        if "id" in record and not record["id"]:
            errors.append({"record": i, "field": "id", "error": "Missing ID"})

        # Validate email format if present
        if "email" in record:
            email = record["email"]
            if "@" not in email or "." not in email:
                errors.append({"record": i, "field": "email", "error": "Invalid email format"})

        # Validate numeric fields
        if "age" in record:
            try:
                age = int(record["age"])
                if age < 0 or age > 150:
                    errors.append({"record": i, "field": "age", "error": "Age out of range"})
            except ValueError:
                errors.append({"record": i, "field": "age", "error": "Age must be numeric"})

        # Validate transaction amount
        if "transaction_amount" in record:
            try:
                amount = float(record["transaction_amount"])
                if amount < 0:
                    errors.append({"record": i, "field": "transaction_amount", "error": "Amount cannot be negative"})
            except ValueError:
                errors.append({"record": i, "field": "transaction_amount", "error": "Amount must be numeric"})

    state["validation_errors"] = errors
    return state


def transform_data(state: ETLState) -> ETLState:
    """Transform data according to business rules."""
    transformed = []

    for i, record in enumerate(state["raw_records"]):
        # Skip records with errors
        record_errors = [e for e in state["validation_errors"] if e["record"] == i]
        if record_errors:
            state["error_count"] += 1
            continue

        transformed_record = record.copy()

        # Apply transformation rules
        if "name" in transformed_record:
            # Normalize name: title case
            transformed_record["name"] = transformed_record["name"].title()

        if "email" in transformed_record:
            # Normalize email: lowercase
            transformed_record["email"] = transformed_record["email"].lower()

        if "age" in transformed_record:
            # Convert age to integer
            transformed_record["age"] = int(transformed_record["age"])

            # Add age group
            age = transformed_record["age"]
            if age < 18:
                transformed_record["age_group"] = "minor"
            elif age < 65:
                transformed_record["age_group"] = "adult"
            else:
                transformed_record["age_group"] = "senior"

        if "transaction_amount" in transformed_record:
            # Convert to float
            transformed_record["transaction_amount"] = float(transformed_record["transaction_amount"])

            # Add transaction tier
            amount = transformed_record["transaction_amount"]
            if amount < 50:
                transformed_record["tier"] = "small"
            elif amount < 200:
                transformed_record["tier"] = "medium"
            else:
                transformed_record["tier"] = "large"

        if "price" in transformed_record and "quantity" in transformed_record:
            # Calculate total value
            price = float(transformed_record["price"])
            quantity = int(transformed_record["quantity"])
            transformed_record["total_value"] = price * quantity

        transformed.append(transformed_record)
        state["success_count"] += 1

    state["transformed_records"] = transformed
    return state


def enrich_data(state: ETLState) -> ETLState:
    """Enrich data with additional information."""
    import datetime

    for record in state["transformed_records"]:
        # Add processing timestamp
        record["processed_at"] = datetime.datetime.now().isoformat()

        # Add data lineage
        record["source"] = state["source_type"]

        # Add data quality score
        error_count = len([e for e in state["validation_errors"] if e.get("record") == record.get("id")])
        record["quality_score"] = 1.0 if error_count == 0 else 0.5

    return state


def load_data(state: ETLState) -> ETLState:
    """Load transformed data to destination."""
    destination = state["destination_type"]

    # Simulate data loading
    if destination == "database":
        # Simulate database insert
        state["load_status"] = f"Loaded {len(state['transformed_records'])} records to database"
    elif destination == "warehouse":
        # Simulate data warehouse load
        state["load_status"] = f"Loaded {len(state['transformed_records'])} records to warehouse"
    elif destination == "file":
        # Simulate file write
        state["load_status"] = f"Written {len(state['transformed_records'])} records to file"

    return state


def generate_stats(state: ETLState) -> ETLState:
    """Generate pipeline statistics."""
    state["pipeline_stats"] = {
        "total_records": state["record_count"],
        "successful": state["success_count"],
        "failed": state["error_count"],
        "success_rate": (state["success_count"] / state["record_count"] * 100) if state["record_count"] > 0 else 0,
        "validation_errors": len(state["validation_errors"]),
    }

    return state


def route_after_validation(state: ETLState) -> Literal["transform", "stats"]:
    """Route based on validation results."""
    # If all records failed validation, skip transformation
    if len(state["validation_errors"]) >= state["record_count"]:
        state["load_status"] = "FAILED: All records invalid"
        return "stats"
    else:
        return "transform"


# Build the graph
workflow = StateGraph(ETLState)

# Add nodes
workflow.add_node("extract", extract_data)
workflow.add_node("validate", validate_records)
workflow.add_node("transform", transform_data)
workflow.add_node("enrich", enrich_data)
workflow.add_node("load", load_data)
workflow.add_node("stats", generate_stats)

# Add edges
workflow.set_entry_point("extract")
workflow.add_edge("extract", "validate")

workflow.add_conditional_edges(
    "validate",
    route_after_validation,
    {
        "transform": "transform",
        "stats": "stats",
    }
)

workflow.add_edge("transform", "enrich")
workflow.add_edge("enrich", "load")
workflow.add_edge("load", "stats")
workflow.add_edge("stats", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the pipeline
    test_pipeline = {
        "source_type": "database",
        "source_config": {},
        "raw_records": [],
        "transformed_records": [],
        "validation_errors": [],
        "record_count": 0,
        "success_count": 0,
        "error_count": 0,
        "transformation_rules": [],
        "destination_type": "warehouse",
        "load_status": "",
        "pipeline_stats": {},
    }

    result = app.invoke(test_pipeline)
    print(f"Pipeline Stats:")
    print(f"  Source: {result['source_type']}")
    print(f"  Destination: {result['destination_type']}")
    print(f"  Total records: {result['pipeline_stats']['total_records']}")
    print(f"  Successful: {result['pipeline_stats']['successful']}")
    print(f"  Failed: {result['pipeline_stats']['failed']}")
    print(f"  Success rate: {result['pipeline_stats']['success_rate']:.1f}%")
    print(f"  Validation errors: {result['pipeline_stats']['validation_errors']}")
    print(f"\nLoad status: {result['load_status']}")
