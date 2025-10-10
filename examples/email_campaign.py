"""
Real-world example: Email Campaign Workflow
Automated email campaign with personalization, scheduling, and delivery tracking.
"""

from typing import TypedDict, Literal, List, Dict
from langgraph.graph import StateGraph, END


class CampaignState(TypedDict):
    """State for email campaign workflow."""
    campaign_id: str
    campaign_type: str  # newsletter, promotional, transactional
    recipients: List[Dict[str, str]]
    segment_criteria: Dict[str, any]
    template_id: str
    personalization_data: Dict[str, any]
    subject_line: str
    email_content: str
    scheduled_time: str
    delivery_status: Dict[str, int]  # sent, failed, bounced
    engagement_metrics: Dict[str, int]  # opened, clicked, unsubscribed
    ab_test_enabled: bool
    ab_variants: List[str]


def load_recipients(state: CampaignState) -> CampaignState:
    """Load recipient list based on segment criteria."""
    criteria = state["segment_criteria"]

    # Simulate database query based on criteria
    # In real system, would query user database with filters

    all_users = [
        {"email": "user1@example.com", "name": "Alice", "segment": "active", "subscribed": True},
        {"email": "user2@example.com", "name": "Bob", "segment": "inactive", "subscribed": True},
        {"email": "user3@example.com", "name": "Charlie", "segment": "active", "subscribed": False},
        {"email": "user4@example.com", "name": "Diana", "segment": "active", "subscribed": True},
    ]

    recipients = []

    for user in all_users:
        # Check if user matches criteria
        matches = True

        if "segment" in criteria and user["segment"] != criteria["segment"]:
            matches = False

        if "subscribed" in criteria and user["subscribed"] != criteria["subscribed"]:
            matches = False

        if matches:
            recipients.append(user)

    state["recipients"] = recipients

    return state


def validate_recipients(state: CampaignState) -> CampaignState:
    """Validate recipient email addresses and subscription status."""
    valid_recipients = []

    for recipient in state["recipients"]:
        # Validate email format
        email = recipient.get("email", "")
        if "@" not in email or "." not in email:
            continue

        # Check subscription status
        if not recipient.get("subscribed", False):
            continue

        # Check bounce history (simplified)
        # In real system, would check against bounce/complaint lists
        if "bounced" not in email:  # Simple check
            valid_recipients.append(recipient)

    state["recipients"] = valid_recipients

    return state


def personalize_content(state: CampaignState) -> CampaignState:
    """Personalize email content for recipients."""
    template = state["email_content"]

    # For each recipient, create personalized version
    # In real system, would use template engine like Jinja2

    personalized_emails = []

    for recipient in state["recipients"]:
        # Simple variable substitution
        content = template
        content = content.replace("{{name}}", recipient.get("name", "Valued Customer"))
        content = content.replace("{{email}}", recipient.get("email", ""))

        # Add personalization data
        if "last_purchase" in recipient:
            content = content.replace("{{last_purchase}}", recipient["last_purchase"])

        recipient["personalized_content"] = content

    return state


def ab_test_subject_lines(state: CampaignState) -> CampaignState:
    """Create A/B test variants for subject lines."""
    if not state["ab_test_enabled"]:
        return state

    # Create variants
    base_subject = state["subject_line"]

    variants = [
        base_subject,  # Control
        f"{base_subject} 🎁",  # Variant A: with emoji
        f"URGENT: {base_subject}",  # Variant B: urgency
    ]

    state["ab_variants"] = variants

    # Assign recipients to variants
    for i, recipient in enumerate(state["recipients"]):
        variant_index = i % len(variants)
        recipient["subject_variant"] = variants[variant_index]
        recipient["variant_name"] = f"variant_{chr(65 + variant_index)}"  # A, B, C

    return state


def schedule_delivery(state: CampaignState) -> CampaignState:
    """Schedule email delivery based on optimal send times."""
    import datetime

    # Determine optimal send time
    campaign_type = state["campaign_type"]

    # Default to immediate send
    send_time = datetime.datetime.now()

    # Adjust based on campaign type
    if campaign_type == "newsletter":
        # Schedule for Tuesday 10 AM
        send_time = send_time.replace(hour=10, minute=0, second=0)
        if send_time.weekday() != 1:  # Not Tuesday
            days_ahead = 1 - send_time.weekday()
            if days_ahead <= 0:
                days_ahead += 7
            send_time = send_time + datetime.timedelta(days=days_ahead)

    elif campaign_type == "promotional":
        # Schedule for afternoon
        send_time = send_time.replace(hour=14, minute=0, second=0)

    state["scheduled_time"] = send_time.isoformat()

    return state


def send_emails(state: CampaignState) -> CampaignState:
    """Send emails to recipients."""
    sent = 0
    failed = 0

    for recipient in state["recipients"]:
        # Simulate email sending
        # In real system, would use email service (SendGrid, AWS SES, etc.)

        email = recipient.get("email", "")

        # Simulate 5% failure rate
        import random
        if random.random() > 0.95:
            failed += 1
        else:
            sent += 1

    state["delivery_status"] = {
        "sent": sent,
        "failed": failed,
        "bounced": 0,  # Would be updated by bounce webhooks
    }

    return state


def track_engagement(state: CampaignState) -> CampaignState:
    """Initialize engagement tracking."""
    # In real system, would set up tracking pixels and click tracking

    state["engagement_metrics"] = {
        "delivered": state["delivery_status"]["sent"],
        "opened": 0,  # Updated via tracking pixel
        "clicked": 0,  # Updated via click tracking
        "unsubscribed": 0,  # Updated via unsubscribe links
    }

    return state


def route_after_validation(state: CampaignState) -> Literal["personalize", END]:
    """Check if we have valid recipients."""
    if len(state["recipients"]) == 0:
        state["delivery_status"] = {"sent": 0, "failed": 0, "bounced": 0}
        return END
    else:
        return "personalize"


# Build the graph
workflow = StateGraph(CampaignState)

# Add nodes
workflow.add_node("load_recipients", load_recipients)
workflow.add_node("validate", validate_recipients)
workflow.add_node("personalize", personalize_content)
workflow.add_node("ab_test", ab_test_subject_lines)
workflow.add_node("schedule", schedule_delivery)
workflow.add_node("send", send_emails)
workflow.add_node("track", track_engagement)

# Add edges
workflow.set_entry_point("load_recipients")
workflow.add_edge("load_recipients", "validate")

workflow.add_conditional_edges(
    "validate",
    route_after_validation,
    {
        "personalize": "personalize",
        END: END,
    }
)

workflow.add_edge("personalize", "ab_test")
workflow.add_edge("ab_test", "schedule")
workflow.add_edge("schedule", "send")
workflow.add_edge("send", "track")
workflow.add_edge("track", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_campaign = {
        "campaign_id": "CAMP_2024_01",
        "campaign_type": "newsletter",
        "recipients": [],
        "segment_criteria": {"segment": "active", "subscribed": True},
        "template_id": "TEMPLATE_001",
        "personalization_data": {},
        "subject_line": "Your Weekly Newsletter",
        "email_content": "Hi {{name}},\n\nHere's what's new this week...",
        "scheduled_time": "",
        "delivery_status": {},
        "engagement_metrics": {},
        "ab_test_enabled": True,
        "ab_variants": [],
    }

    result = app.invoke(test_campaign)
    print(f"Campaign ID: {result['campaign_id']}")
    print(f"Type: {result['campaign_type']}")
    print(f"Recipients: {len(result['recipients'])}")
    print(f"Scheduled: {result['scheduled_time']}")
    print(f"\nDelivery Status:")
    print(f"  Sent: {result['delivery_status'].get('sent', 0)}")
    print(f"  Failed: {result['delivery_status'].get('failed', 0)}")
    print(f"\nA/B Test Variants: {len(result['ab_variants'])}")
    if result['ab_variants']:
        for variant in result['ab_variants']:
            print(f"  - {variant}")
