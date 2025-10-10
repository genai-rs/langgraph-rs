"""
Real-world example: Recommendation Engine
Personalized product recommendations with fallback strategies.
"""

from typing import TypedDict, Literal, List, Dict
from langgraph.graph import StateGraph, END


class RecommendationState(TypedDict):
    """State for recommendation workflow."""
    user_id: str
    user_profile: Dict[str, any]
    context: Dict[str, str]  # page, category, search_query, etc.
    browsing_history: List[str]
    purchase_history: List[str]
    recommendations: List[Dict[str, any]]
    recommendation_strategy: str
    confidence_score: float
    personalization_level: str  # high, medium, low, none


def fetch_user_profile(state: RecommendationState) -> RecommendationState:
    """Fetch user profile and history."""
    user_id = state["user_id"]

    # Simulate database lookup
    # In real system, would query user service/database

    if user_id:
        # Simulate user profile
        state["user_profile"] = {
            "age_group": "25-34",
            "interests": ["technology", "books", "fitness"],
            "spending_tier": "medium",
        }

        # Simulate browsing history
        state["browsing_history"] = ["product_123", "product_456", "product_789"]

        # Simulate purchase history
        state["purchase_history"] = ["product_100", "product_200"]

        state["personalization_level"] = "high"
    else:
        # Anonymous user
        state["user_profile"] = {}
        state["browsing_history"] = []
        state["purchase_history"] = []
        state["personalization_level"] = "none"

    return state


def collaborative_filtering(state: RecommendationState) -> RecommendationState:
    """Generate recommendations using collaborative filtering."""
    recommendations = []

    # Simulate collaborative filtering algorithm
    # In real system, would use matrix factorization, nearest neighbors, etc.

    if state["purchase_history"]:
        # Find similar users and their purchases
        similar_user_purchases = ["product_101", "product_201", "product_301"]

        for product_id in similar_user_purchases[:5]:
            recommendations.append({
                "product_id": product_id,
                "score": 0.8,
                "reason": "Users who bought similar items also bought this",
            })

    state["recommendations"] = recommendations
    state["recommendation_strategy"] = "collaborative"
    state["confidence_score"] = 0.8 if recommendations else 0.0

    return state


def content_based_filtering(state: RecommendationState) -> RecommendationState:
    """Generate recommendations using content-based filtering."""
    recommendations = []

    # Simulate content-based filtering
    # In real system, would use item features, TF-IDF, embeddings, etc.

    if state["browsing_history"]:
        # Find similar items based on content
        similar_items = ["product_124", "product_457", "product_790"]

        for product_id in similar_items[:5]:
            recommendations.append({
                "product_id": product_id,
                "score": 0.7,
                "reason": "Similar to items you viewed",
            })

    state["recommendations"] = recommendations
    state["recommendation_strategy"] = "content_based"
    state["confidence_score"] = 0.7 if recommendations else 0.0

    return state


def contextual_recommendations(state: RecommendationState) -> RecommendationState:
    """Generate context-aware recommendations."""
    recommendations = []

    context = state["context"]

    # Use context to generate recommendations
    if "category" in context:
        category = context["category"]
        # Top items in this category
        category_items = [f"cat_{category}_{i}" for i in range(1, 6)]

        for product_id in category_items:
            recommendations.append({
                "product_id": product_id,
                "score": 0.6,
                "reason": f"Popular in {category}",
            })

    elif "search_query" in context:
        query = context["search_query"]
        # Search-based recommendations
        search_items = [f"search_{i}" for i in range(1, 6)]

        for product_id in search_items:
            recommendations.append({
                "product_id": product_id,
                "score": 0.5,
                "reason": f"Matches '{query}'",
            })

    state["recommendations"] = recommendations
    state["recommendation_strategy"] = "contextual"
    state["confidence_score"] = 0.6 if recommendations else 0.0

    return state


def trending_items(state: RecommendationState) -> RecommendationState:
    """Fallback to trending/popular items."""
    # Simulate trending items query
    # In real system, would query analytics/popularity metrics

    trending = [
        {"product_id": "trending_1", "score": 0.9, "reason": "Trending now"},
        {"product_id": "trending_2", "score": 0.85, "reason": "Trending now"},
        {"product_id": "trending_3", "score": 0.8, "reason": "Trending now"},
        {"product_id": "trending_4", "score": 0.75, "reason": "Trending now"},
        {"product_id": "trending_5", "score": 0.7, "reason": "Trending now"},
    ]

    state["recommendations"] = trending
    state["recommendation_strategy"] = "trending"
    state["confidence_score"] = 0.4  # Lower confidence for generic recommendations

    return state


def rank_and_diversify(state: RecommendationState) -> RecommendationState:
    """Rank and diversify final recommendations."""
    recs = state["recommendations"]

    # Sort by score
    recs.sort(key=lambda x: x["score"], reverse=True)

    # Take top 10
    state["recommendations"] = recs[:10]

    # Update confidence based on diversity
    unique_strategies = len(set(r.get("reason", "") for r in recs))
    if unique_strategies > 3:
        state["confidence_score"] = min(state["confidence_score"] + 0.1, 1.0)

    return state


def route_after_profile(state: RecommendationState) -> Literal["collaborative", "contextual"]:
    """Route based on personalization level."""
    if state["personalization_level"] in ["high", "medium"]:
        if state["purchase_history"]:
            return "collaborative"
        else:
            return "contextual"
    else:
        return "contextual"


def route_after_strategy(state: RecommendationState) -> Literal["rank", "content_based", "trending"]:
    """Route based on recommendation quality."""
    if state["confidence_score"] >= 0.7:
        return "rank"
    elif state["browsing_history"]:
        return "content_based"
    else:
        return "trending"


# Build the graph
workflow = StateGraph(RecommendationState)

# Add nodes
workflow.add_node("profile", fetch_user_profile)
workflow.add_node("collaborative", collaborative_filtering)
workflow.add_node("content_based", content_based_filtering)
workflow.add_node("contextual", contextual_recommendations)
workflow.add_node("trending", trending_items)
workflow.add_node("rank", rank_and_diversify)

# Add edges
workflow.set_entry_point("profile")

workflow.add_conditional_edges(
    "profile",
    route_after_profile,
    {
        "collaborative": "collaborative",
        "contextual": "contextual",
    }
)

workflow.add_conditional_edges(
    "collaborative",
    route_after_strategy,
    {
        "rank": "rank",
        "content_based": "content_based",
        "trending": "trending",
    }
)

workflow.add_conditional_edges(
    "contextual",
    route_after_strategy,
    {
        "rank": "rank",
        "content_based": "content_based",
        "trending": "trending",
    }
)

workflow.add_edge("content_based", "rank")
workflow.add_edge("trending", "rank")
workflow.add_edge("rank", END)

# Compile the graph
app = workflow.compile()


if __name__ == "__main__":
    # Test the workflow
    test_cases = [
        {
            "user_id": "user_123",
            "user_profile": {},
            "context": {"page": "home"},
            "browsing_history": [],
            "purchase_history": [],
            "recommendations": [],
            "recommendation_strategy": "",
            "confidence_score": 0.0,
            "personalization_level": "",
        },
        {
            "user_id": "",  # Anonymous user
            "user_profile": {},
            "context": {"category": "electronics"},
            "browsing_history": [],
            "purchase_history": [],
            "recommendations": [],
            "recommendation_strategy": "",
            "confidence_score": 0.0,
            "personalization_level": "",
        },
    ]

    for test in test_cases:
        result = app.invoke(test)
        print(f"\nUser ID: {test['user_id'] or 'Anonymous'}")
        print(f"Strategy: {result['recommendation_strategy']}")
        print(f"Confidence: {result['confidence_score']:.2f}")
        print(f"Recommendations: {len(result['recommendations'])} items")
        for rec in result['recommendations'][:3]:
            print(f"  - {rec['product_id']}: {rec['reason']}")
