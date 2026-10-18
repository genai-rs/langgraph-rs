# LangGraph-RS Examples

This directory contains Python scripts illustrating LangGraph patterns. The current CLI cannot extract their graph metadata or convert their behavior to Rust. The scripts and their descriptions are not verified Rust conversions or production-ready applications.

## Example Categories

### 1. Customer Support & Chatbots

#### Customer Support Agent (`customer_support_agent.py`)
**Pattern**: Multi-tier routing with escalation
**Features**:
- Sentiment analysis
- Category classification (technical, billing, general)
- Priority-based routing (tier1 → tier2 → escalate)
- Conditional escalation based on complexity

**Use Cases**: Customer service ***REMOVED***, support ticket routing, helpdesk systems

#### Chatbot with Fallback (`chatbot_with_fallback.py`)
**Pattern**: Intent detection with graceful degradation
**Features**:
- Intent classification (greeting, question, command)
- Confidence-based routing
- Multiple fallback strategies
- Automatic escalation to human after repeated fallbacks

**Use Cases**: Conversational AI, virtual assistants, FAQ bots

### 2. Content Processing

#### Content Moderation System (`content_moderation.py`)
**Pattern**: Multi-stage filtering pipeline
**Features**:
- Keyword filtering
- Sentiment/toxicity analysis
- ML-based classification
- Auto-moderation with human review queue

**Use Cases**: Social media moderation, comment filtering, UGC platforms

#### Document Processing (`document_processing.py`)
**Pattern**: ETL with validation and retry
**Features**:
- Document type detection
- Text extraction
- Metadata parsing
- Entity recognition
- Validation with retry logic

**Use Cases**: Document management, OCR pipelines, data extraction

### 3. Data Processing

#### Data Validation & Enrichment (`data_validation_enrichment.py`)
**Pattern**: Multi-stage data quality pipeline
**Features**:
- Schema validation
- Data enrichment
- Quality scoring
- Manual review flagging

**Use Cases**: Data ingestion, ETL preprocessing, data quality management

#### ETL Pipeline (`etl_pipeline.py`)
**Pattern**: Extract-Transform-Load with error handling
**Features**:
- Multi-source extraction (database, API, file)
- Comprehensive validation
- Data transformation and enrichment
- Quality metrics and statistics

**Use Cases**: Data warehousing, analytics pipelines, batch processing

### 4. E-Commerce & Business

#### Order Fulfillment (`order_fulfillment.py`)
**Pattern**: End-to-end transaction workflow
**Features**:
- Order validation
- Inventory checking
- Payment processing
- Shipping method selection
- Cancellation and refund logic

**Use Cases**: E-commerce platforms, order management systems, fulfillment centers

#### Recommendation Engine (`recommendation_engine.py`)
**Pattern**: Personalized recommendations with fallback
**Features**:
- User profiling
- Collaborative filtering
- Content-based filtering
- Contextual recommendations
- Trending items fallback

**Use Cases**: Product recommendations, content discovery, personalization engines

### 5. Infrastructure & Routing

#### API Router with Retry (`api_router_with_retry.py`)
**Pattern**: Intelligent routing with circuit breaker
**Features**:
- Service routing
- Cache checking
- Retry logic with backoff
- Circuit breaker pattern
- Fallback service

**Use Cases**: API gateways, microservice routing, load balancing

#### A/B Testing Router (`ab_testing_router.py`)
**Pattern**: Experiment management and routing
**Features**:
- User segmentation
- Variant assignment
- Traffic allocation
- Feature flag management
- Analytics tracking

**Use Cases**: Experimentation platforms, feature flags, growth optimization

### 6. Marketing Automation

#### Email Campaign (`email_campaign.py`)
**Pattern**: Automated campaign workflow
**Features**:
- Recipient segmentation
- Email validation
- Content personalization
- A/B testing for subject lines
- Delivery scheduling
- Engagement tracking

**Use Cases**: Email marketing, newsletter distribution, transactional emails

## Example Structure

The `.py` files are illustrative LangGraph sources and may contain TODOs or simplified logic. Only two Rust scaffold files are present: `customer_support_agent_generated.rs` and `simple_workflow_generated.rs`. They are not produced by the current CLI and do not establish behavioral equivalence. Their node and routing logic must be reviewed and implemented.

## Running Examples

There is no supported CLI command to run or convert these examples yet. Read the Python source directly; do not use the old generated-example `cargo test --example` command, which does not correspond to a Cargo example target in this repository. The CLI status and supported commands are documented in the [main README](../README.md).

## Conversion Patterns

### Pattern 1: Linear Workflow
**Example**: Document Processing
- Sequential node execution
- No conditional branching
- Error handling at each stage

### Pattern 2: Conditional Branching
**Example**: Customer Support Agent
- Multi-path routing based on conditions
- Dynamic path selection
- Convergence points

### Pattern 3: Retry & Fallback
**Example**: API Router with Retry
- Retry logic with backoff
- Circuit breaker pattern
- Graceful degradation to fallback

### Pattern 4: Loops & Iteration
**Example**: Order Fulfillment
- Loop-back on certain conditions
- Counter-based termination
- State accumulation

### Pattern 5: Multi-Stage Pipeline
**Example**: ETL Pipeline
- Sequential transformation stages
- Quality gates between stages
- Error collection and reporting

## Performance

No reproducible Python-versus-Rust benchmark of equivalent workflow behavior is available. The example scripts have not been converted into working Rust implementations, so this directory makes no performance claim. See [the performance guide](../docs/performance.md) for the limits of the existing code-generation timing tests.

## Type Mappings

Illustrative Python-to-Rust type correspondences. This table is not a compatibility guarantee or evidence of a working end-to-end conversion:

```python
# Python TypedDict
class State(TypedDict):
    messages: list[str]
    context: dict[str, str]
    counter: int
    confidence: float
    enabled: bool
    optional_field: str | None
```

```rust
// Illustrative Rust state shape; not generated by the current CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub messages: Vec<String>,
    pub context: HashMap<String, String>,
    pub counter: i64,
    pub confidence: f64,
    pub enabled: bool,
    pub optional_field: Option<String>,
}
```

## Best Practices

### 1. State Design
- Keep state flat when possible
- Use strongly-typed fields
- Avoid `Any` types (use serde_json::Value sparingly)

### 2. Node Functions
- Make nodes pure functions when possible
- Handle errors explicitly
- Use async for I/O operations
- Keep business logic testable

### 3. Routing Logic
- Keep routing functions simple
- Use pattern matching in Rust
- Document routing conditions
- Test all branches

### 4. Error Handling
- Use Result types consistently
- Provide context with anyhow
- Don't panic in production code
- Log errors appropriately

## Contributing Examples

To add a new example:

1. Add a clearly identified Python source example and document any TODOs or simplifications.
2. Add tests for the Python example if it is intended to be executable.
3. If adding Rust, label it as a scaffold unless equivalent behavior is implemented and tested.
4. Update this README without claiming automatic conversion or performance results.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

## Additional Resources

- [Migration Guide](../docs/migration-guide.md) - Current migration limits and manual guidance
- [Performance Guide](../docs/performance.md) - Benchmark status and requirements for future claims
- [Deployment Guide](../docs/deployment.md) - Deployment examples; not verified for generated workflows
- [LangGraph Docs](https://langchain-ai.github.io/langgraph/) - Original framework

## License

These examples are dual-licensed under MIT OR Apache-2.0, same as the main project.
