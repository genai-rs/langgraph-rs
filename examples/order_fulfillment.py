"""
Real-world example: Order Fulfillment Workflow
End-to-end order processing with inventory, payment, and shipping.
"""

from typing import TypedDict, Literal, List, Dict
from langgraph.graph import StateGraph, END


class OrderState(TypedDict):
    """State for order fulfillment workflow."""
    order_id: str
    customer_id: str
    items: List[Dict[str, any]]
    total_amount: float
    inventory_available: bool
    payment_status: str  # pending, authorized, captured, failed
    shipping_method: str
    shipping_address: str
    tracking_number: str
    order_status: str  # created, processing, shipped, delivered, cancelled
    notes: List[str]


def validate_order(state: OrderState) -> OrderState:
    """Validate order details."""
    notes = []

    # Check required fields
    if not state["customer_id"]:
        notes.append("ERROR: Missing customer ID")

    if not state["items"] or len(state["items"]) == 0:
        notes.append("ERROR: No items in order")

    if not state["shipping_address"]:
        notes.append("ERROR: Missing shipping address")

    # Calculate total if not provided
    if state["total_amount"] == 0 and state["items"]:
        total = sum(item.get("price", 0) * item.get("quantity", 1) for item in state["items"])
        state["total_amount"] = total
        notes.append(f"Calculated total: ${total:.2f}")

    state["notes"].extend(notes)
    return state


def check_inventory(state: OrderState) -> OrderState:
    """Check inventory availability for all items."""
    # Simulate inventory check
    all_available = True

    for item in state["items"]:
        item_id = item.get("product_id", "")
        quantity = item.get("quantity", 0)

        # Simulate inventory database lookup
        # In real system, would query inventory service
        available_stock = 100  # Simplified

        if quantity > available_stock:
            all_available = False
            state["notes"].append(f"Insufficient stock for item {item_id}")

    state["inventory_available"] = all_available

    if all_available:
        # Reserve inventory
        state["notes"].append("Inventory reserved")

    return state


def process_payment(state: OrderState) -> OrderState:
    """Process payment for the order."""
    amount = state["total_amount"]

    # Simulate payment processing
    # In real system, would integrate with Stripe, PayPal, etc.

    if amount <= 0:
        state["payment_status"] = "failed"
        state["notes"].append("ERROR: Invalid payment amount")
    elif amount > 10000:
        # High-value orders need manual review
        state["payment_status"] = "pending"
        state["notes"].append("High-value order flagged for review")
    else:
        # Simulate successful payment
        state["payment_status"] = "captured"
        state["notes"].append(f"Payment captured: ${amount:.2f}")

    return state


def select_shipping_method(state: OrderState) -> OrderState:
    """Select appropriate shipping method based on order details."""
    total = state["total_amount"]
    item_count = len(state["items"])

    # Business logic for shipping selection
    if total >= 100:
        state["shipping_method"] = "free_standard"
        state["notes"].append("Free shipping applied (order > $100)")
    elif item_count > 5:
        state["shipping_method"] = "bulk_shipping"
        state["notes"].append("Bulk shipping selected")
    elif total >= 50:
        state["shipping_method"] = "standard"
    else:
        state["shipping_method"] = "economy"

    return state


def create_shipment(state: OrderState) -> OrderState:
    """Create shipment and generate tracking number."""
    # Simulate shipment creation
    # In real system, would integrate with shipping provider API

    import uuid
    tracking_number = f"TRK{uuid.uuid4().hex[:8].upper()}"

    state["tracking_number"] = tracking_number
    state["order_status"] = "shipped"
    state["notes"].append(f"Shipment created: {tracking_number}")

    return state


def cancel_order(state: OrderState) -> OrderState:
    """Cancel order and reverse any transactions."""
    state["order_status"] = "cancelled"

    # Reverse payment if captured
    if state["payment_status"] == "captured":
        state["payment_status"] = "refunded"
        state["notes"].append("Payment refunded")

    # Release inventory if reserved
    if state["inventory_available"]:
        state["notes"].append("Inventory released")

    return state


def finalize_order(state: OrderState) -> OrderState:
    """Mark order as complete."""
    state["order_status"] = "processing"
    return state


def route_after_validation(state: OrderState) -> Literal["check_inventory", "cancel"]:
    """Route based on validation results."""
    # Check for critical errors
    error_notes = [n for n in state["notes"] if n.startswith("ERROR:")]

    if error_notes:
        return "cancel"
    else:
        return "check_inventory"


def route_after_inventory(state: OrderState) -> Literal["payment", "cancel"]:
    """Route based on inventory availability."""
    if state["inventory_available"]:
        return "payment"
    else:
        return "cancel"


def route_after_payment(state: OrderState) -> Literal["shipping_select", "cancel"]:
    """Route based on payment status."""
    if state["payment_status"] == "captured":
        return "shipping_select"
    else:
        return "cancel"


# Build the graph
workflow = StateGraph(OrderState)

# Add nodes
workflow.add_node("validate", validate_order)
workflow.add_node("check_inventory", check_inventory)
workflow.add_node("payment", process_payment)
workflow.add_node("shipping_select", select_shipping_method)
workflow.add_node("shipment", create_shipment)
workflow.add_node("cancel", cancel_order)
workflow.add_node("finalize", finalize_order)

# Add edges
workflow.set_entry_point("validate")

workflow.add_conditional_edges(
    "validate",
    route_after_validation,
    {
        "check_inventory": "check_inventory",
        "cancel": "cancel",
    }
)

workflow.add_conditional_edges(
    "check_inventory",
    route_after_inventory,
    {
        "payment": "payment",
        "cancel": "cancel",
    }
)

workflow.add_conditional_edges(
    "payment",
    route_after_payment,
    {
        "shipping_select": "shipping_select",
        "cancel": "cancel",
    }
)

workflow.add_edge("shipping_select", "shipment")
workflow.add_edge("shipment", "finalize")
workflow.add_edge("finalize", END)
workflow.add_edge("cancel", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_order = {
        "order_id": "ORD123",
        "customer_id": "CUST456",
        "items": [
            {"product_id": "P1", "name": "Widget", "price": 29.99, "quantity": 2},
            {"product_id": "P2", "name": "Gadget", "price": 49.99, "quantity": 1},
        ],
        "total_amount": 0.0,  # Will be calculated
        "inventory_available": False,
        "payment_status": "pending",
        "shipping_method": "",
        "shipping_address": "123 Main St, City, State 12345",
        "tracking_number": "",
        "order_status": "created",
        "notes": [],
    }

    result = app.invoke(test_order)
    print(f"Order ID: {result['order_id']}")
    print(f"Total: ${result['total_amount']:.2f}")
    print(f"Status: {result['order_status']}")
    print(f"Payment: {result['payment_status']}")
    print(f"Shipping: {result['shipping_method']}")
    print(f"Tracking: {result['tracking_number']}")
    print("\nNotes:")
    for note in result["notes"]:
        print(f"  - {note}")
