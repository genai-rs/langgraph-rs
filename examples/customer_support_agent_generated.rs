// Generated from customer_support_agent.py
// Real-world example: Customer Support Agent
// Handles customer queries with routing to different support tiers

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// State for customer support workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportState {
    pub customer_message: String,
    pub sentiment: String,  // positive, neutral, negative
    pub category: String,   // technical, billing, general
    pub priority: String,   // low, medium, high
    pub response: String,
    pub escalated: bool,
    pub resolved: bool,
}

impl SupportState {
    pub fn new(customer_message: String) -> Self {
        Self {
            customer_message,
            sentiment: String::new(),
            category: String::new(),
            priority: String::new(),
            response: String::new(),
            escalated: false,
            resolved: false,
        }
    }
}

/// Analyze customer message for sentiment and category
async fn analyze_message(mut state: SupportState) -> Result<SupportState> {
    let message = state.customer_message.to_lowercase();

    // Simple sentiment analysis
    state.sentiment = if message.contains("angry")
        || message.contains("frustrated")
        || message.contains("terrible") {
        "negative".to_string()
    } else if message.contains("great")
        || message.contains("happy")
        || message.contains("excellent") {
        "positive".to_string()
    } else {
        "neutral".to_string()
    };

    // Simple categorization
    state.category = if message.contains("bug")
        || message.contains("error")
        || message.contains("crash")
        || message.contains("technical") {
        "technical".to_string()
    } else if message.contains("payment")
        || message.contains("billing")
        || message.contains("charge")
        || message.contains("refund") {
        "billing".to_string()
    } else {
        "general".to_string()
    };

    // Set priority based on sentiment and category
    state.priority = if state.sentiment == "negative"
        && (state.category == "technical" || state.category == "billing") {
        "high".to_string()
    } else if state.sentiment == "negative" || state.category == "billing" {
        "medium".to_string()
    } else {
        "low".to_string()
    };

    Ok(state)
}

/// Tier 1 support - handles simple queries
async fn handle_tier1(mut state: SupportState) -> Result<SupportState> {
    if state.category == "general" && state.priority == "low" {
        let preview = state.customer_message.chars().take(50).collect::<String>();
        state.response = format!("Thank you for contacting us about: {}...", preview);
        state.resolved = true;
    } else {
        state.resolved = false;
    }
    Ok(state)
}

/// Tier 2 support - handles technical and billing issues
async fn handle_tier2(mut state: SupportState) -> Result<SupportState> {
    if (state.category == "technical" || state.category == "billing")
        && (state.priority == "low" || state.priority == "medium") {
        state.response = format!("Our specialist will handle your {} issue.", state.category);
        state.resolved = true;
    } else {
        state.resolved = false;
    }
    Ok(state)
}

/// Escalate to senior support
async fn escalate(mut state: SupportState) -> Result<SupportState> {
    state.response = "This has been escalated to our senior support team.".to_string();
    state.escalated = true;
    state.resolved = true;
    Ok(state)
}

/// Route to appropriate support tier based on analysis
fn route_after_analysis(state: &SupportState) -> &'static str {
    if state.priority == "high" {
        "escalate"
    } else if state.category == "technical" || state.category == "billing" {
        "tier2"
    } else {
        "tier1"
    }
}

/// Route from tier1 based on resolution
fn route_after_tier1(state: &SupportState) -> &'static str {
    if state.resolved {
        "end"
    } else {
        "tier2"
    }
}

/// Route from tier2 based on resolution
fn route_after_tier2(state: &SupportState) -> &'static str {
    if state.resolved {
        "end"
    } else {
        "escalate"
    }
}

/// Execute the customer support workflow
pub async fn execute_graph(mut state: SupportState) -> Result<SupportState> {
    // Start at analyze node
    state = analyze_message(state)
        .await
        .context("Failed to analyze message")?;

    let mut current_node = route_after_analysis(&state);

    loop {
        match current_node {
            "tier1" => {
                state = handle_tier1(state)
                    .await
                    .context("Failed at tier1 node")?;
                current_node = route_after_tier1(&state);
            }
            "tier2" => {
                state = handle_tier2(state)
                    .await
                    .context("Failed at tier2 node")?;
                current_node = route_after_tier2(&state);
            }
            "escalate" => {
                state = escalate(state)
                    .await
                    .context("Failed at escalate node")?;
                break;
            }
            "end" => break,
            _ => bail!("Unknown node: {}", current_node),
        }
    }

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_query() {
        let state = SupportState::new("I love your product!".to_string());
        let result = execute_graph(state).await.unwrap();

        assert_eq!(result.sentiment, "positive");
        assert_eq!(result.category, "general");
        assert_eq!(result.priority, "low");
        assert!(result.resolved);
        assert!(!result.escalated);
    }

    #[tokio::test]
    async fn test_critical_bug() {
        let state = SupportState::new("I found a critical bug in the app".to_string());
        let result = execute_graph(state).await.unwrap();

        assert_eq!(result.category, "technical");
        assert!(result.resolved);
    }

    #[tokio::test]
    async fn test_billing_issue() {
        let state = SupportState::new("I was charged twice for my subscription".to_string());
        let result = execute_graph(state).await.unwrap();

        assert_eq!(result.category, "billing");
        assert_eq!(result.priority, "medium");
        assert!(result.resolved);
    }

    #[tokio::test]
    async fn test_high_priority_escalation() {
        let state = SupportState::new("This is terrible! The app crashes constantly!".to_string());
        let result = execute_graph(state).await.unwrap();

        assert_eq!(result.sentiment, "negative");
        assert_eq!(result.category, "technical");
        assert_eq!(result.priority, "high");
        assert!(result.escalated);
    }
}
