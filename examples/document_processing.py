"""
Real-world example: Document Processing Pipeline
Extract, transform, and validate documents with error recovery.
"""

from typing import TypedDict, Literal, Dict, List
from langgraph.graph import StateGraph, END


class DocumentState(TypedDict):
    """State for document processing workflow."""
    raw_content: str
    document_type: str  # pdf, docx, txt, email
    extracted_text: str
    metadata: Dict[str, str]
    entities: List[Dict[str, str]]  # Named entities
    validation_errors: List[str]
    processed: bool
    retry_count: int


def detect_document_type(state: DocumentState) -> DocumentState:
    """Detect document type from content."""
    content = state["raw_content"]

    if content.startswith("From:") or content.startswith("To:"):
        state["document_type"] = "email"
    elif "Dear" in content and "Sincerely" in content:
        state["document_type"] = "letter"
    elif len(content) < 100:
        state["document_type"] = "note"
    else:
        state["document_type"] = "document"

    return state


def extract_text(state: DocumentState) -> DocumentState:
    """Extract text from document."""
    # In real system, would use OCR, PDF parsing, etc.
    state["extracted_text"] = state["raw_content"]
    return state


def extract_metadata(state: DocumentState) -> DocumentState:
    """Extract metadata from document."""
    metadata = {}
    text = state["extracted_text"]

    # Simple date extraction
    if "date:" in text.lower():
        lines = text.split("\n")
        for line in lines:
            if "date:" in line.lower():
                metadata["date"] = line.split(":")[-1].strip()

    # Email-specific metadata
    if state["document_type"] == "email":
        for line in text.split("\n"):
            if line.startswith("From:"):
                metadata["from"] = line.replace("From:", "").strip()
            elif line.startswith("To:"):
                metadata["to"] = line.replace("To:", "").strip()
            elif line.startswith("Subject:"):
                metadata["subject"] = line.replace("Subject:", "").strip()

    state["metadata"] = metadata
    return state


def extract_entities(state: DocumentState) -> DocumentState:
    """Extract named entities from text."""
    # Simplified NER - real system would use spaCy, transformers, etc.
    entities = []
    text = state["extracted_text"]

    # Look for email addresses
    words = text.split()
    for word in words:
        if "@" in word:
            entities.append({"type": "email", "value": word})

    # Look for capitalized words (simplified name detection)
    for word in words:
        if word and word[0].isupper() and len(word) > 1:
            entities.append({"type": "name", "value": word})

    state["entities"] = entities
    return state


def validate_document(state: DocumentState) -> DocumentState:
    """Validate processed document."""
    errors = []

    # Check for required metadata
    if state["document_type"] == "email":
        if "from" not in state["metadata"]:
            errors.append("Missing 'from' field in email")
        if "to" not in state["metadata"]:
            errors.append("Missing 'to' field in email")

    # Check for extracted text
    if not state["extracted_text"] or len(state["extracted_text"]) < 10:
        errors.append("Insufficient text extracted")

    state["validation_errors"] = errors
    return state


def finalize_document(state: DocumentState) -> DocumentState:
    """Mark document as successfully processed."""
    state["processed"] = True
    return state


def retry_extraction(state: DocumentState) -> DocumentState:
    """Retry extraction with different strategy."""
    state["retry_count"] += 1

    # Simplified retry logic
    if state["retry_count"] <= 2:
        # In real system, would try different extraction method
        state["extracted_text"] = state["raw_content"]
        state["validation_errors"] = []  # Clear errors for retry
    else:
        # Max retries reached
        state["processed"] = False

    return state


def route_after_validation(state: DocumentState) -> Literal["finalize", "retry"]:
    """Route based on validation results."""
    if len(state["validation_errors"]) == 0:
        return "finalize"
    elif state["retry_count"] < 2:
        return "retry"
    else:
        return "finalize"  # Give up after max retries


# Build the graph
workflow = StateGraph(DocumentState)

# Add nodes
workflow.add_node("detect_type", detect_document_type)
workflow.add_node("extract_text", extract_text)
workflow.add_node("extract_metadata", extract_metadata)
workflow.add_node("extract_entities", extract_entities)
workflow.add_node("validate", validate_document)
workflow.add_node("finalize", finalize_document)
workflow.add_node("retry", retry_extraction)

# Add edges
workflow.set_entry_point("detect_type")
workflow.add_edge("detect_type", "extract_text")
workflow.add_edge("extract_text", "extract_metadata")
workflow.add_edge("extract_metadata", "extract_entities")
workflow.add_edge("extract_entities", "validate")

workflow.add_conditional_edges(
    "validate",
    route_after_validation,
    {
        "finalize": "finalize",
        "retry": "retry",
    }
)

workflow.add_edge("retry", "extract_text")  # Loop back
workflow.add_edge("finalize", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_email = """From: john@example.com
To: jane@example.com
Subject: Meeting Tomorrow
Date: 2024-01-15

Hi Jane,

Let's meet tomorrow to discuss the project.

Best,
John"""

    state = {
        "raw_content": test_email,
        "document_type": "",
        "extracted_text": "",
        "metadata": {},
        "entities": [],
        "validation_errors": [],
        "processed": False,
        "retry_count": 0,
    }

    result = app.invoke(state)
    print("Document Type:", result["document_type"])
    print("Metadata:", result["metadata"])
    print("Entities found:", len(result["entities"]))
    print("Processed:", result["processed"])
    print("Errors:", result["validation_errors"])
