# langgraph-rs Development TODO List

## 🎯 Project Vision
Transform LangGraph Python workflows into high-performance Rust implementations for production LLM services.

## Phase 1: Foundation ✅ (Complete)
- [x] Create project structure
- [x] Set up Cargo workspace
- [x] Create basic module structure
- [x] Implement CLI skeleton
- [x] Add example workflows

## Phase 2: Python Introspection 🚧
- [ ] Set up Python test environment
  - [ ] Create requirements.txt with LangGraph dependencies
  - [ ] Set up Python virtual environment scripts
  - [ ] Add test fixtures with various LangGraph patterns
  - [ ] Configure pytest (including async support) and benchmarking plugins
  - [ ] Document fixture expectations in `docs/` for contributors

- [ ] Implement PyO3 introspection
  - [ ] Extract node functions and metadata
  - [ ] Extract edge definitions and routing
  - [ ] Parse state schema from TypedDict
  - [ ] Capture conditional edge functions
  - [ ] Extract tool definitions

- [ ] Create type mapping system
  - [ ] Map Python types to Rust types
  - [ ] Handle complex types (List, Dict, Optional)
  - [ ] Support custom types and dataclasses
  - [ ] Generate proper serde derives

## Phase 3: Code Generation 📝
- [ ] Basic graph generation
  - [ ] Generate state structs
  - [ ] Generate node functions (stubs)
  - [ ] Create graph executor
  - [ ] Add proper error handling
  - [ ] Traverse edges to determine execution order (support branching/loops)
  - [ ] Sanitize node identifiers before emitting Rust symbols
  - [ ] Generate state initialization helpers used by scaffolded tests

- [ ] Advanced features
  - [ ] Conditional routing implementation
  - [ ] Tool node generation
  - [ ] Async/await patterns
  - [ ] Streaming support
  - [ ] Checkpointing/persistence

- [ ] Code quality
  - [ ] Format generated code with rustfmt
  - [ ] Add comprehensive comments
  - [ ] Generate documentation
  - [ ] Include original Python as reference
  - [ ] Add tests covering execution ordering and identifier sanitization

## Phase 4: Runtime Support 🏃
- [ ] Core runtime
  - [ ] Implement state management
  - [ ] Add execution tracing
  - [ ] Error recovery mechanisms
  - [ ] Performance monitoring

- [ ] LLM integrations
  - [ ] OpenAI provider
  - [ ] Anthropic provider
  - [ ] Local model support (Ollama)
  - [ ] Custom provider trait

- [ ] Tool system
  - [ ] HTTP tool implementation
  - [ ] Database tools
  - [ ] File system tools
  - [ ] Custom tool framework

## Phase 5: Validation & Testing 🧪
- [ ] Validation framework
  - [ ] Run Python and Rust in parallel
  - [ ] Compare outputs
  - [ ] Performance benchmarking
  - [ ] Regression testing

- [ ] Test suite
  - [ ] Unit tests for each module
  - [ ] Integration tests
  - [ ] End-to-end workflow tests
  - [ ] Property-based testing

- [ ] Example coverage
  - [ ] Simple linear workflows
  - [ ] Conditional branching
  - [ ] Tool usage
  - [ ] Multi-agent patterns
  - [ ] Streaming scenarios

## Phase 6: Production Features 🚀
- [ ] Performance optimizations
  - [ ] Profile and optimize hot paths
  - [ ] Implement caching
  - [ ] Connection pooling
  - [ ] Batch processing support

- [ ] Deployment tools
  - [ ] Docker containers
  - [ ] Kubernetes manifests
  - [ ] CI/CD pipelines
  - [ ] Monitoring integration

- [ ] Documentation
  - [ ] API documentation
  - [ ] Migration guide
  - [ ] Best practices
  - [ ] Performance tuning guide
  - [ ] Video tutorials

## Phase 7: Ecosystem Integration 🌐
- [ ] Package publishing
  - [ ] Publish to crates.io
  - [ ] GitHub releases
  - [ ] Homebrew formula
  - [ ] Cargo-binstall support

- [ ] Community features
  - [ ] Plugin system
  - [ ] Template library
  - [ ] Online playground
  - [ ] VS Code extension

## Quick Wins 🎯
These can be done in parallel:
- [ ] Add GitHub Actions CI/CD
- [ ] Set up code coverage
- [ ] Create CONTRIBUTING.md
- [ ] Add badges to README
- [ ] Create Discord/Slack community
- [ ] Write blog post announcement
- [ ] Update CLI codegen to reference `langgraph-runtime` via workspace path instead of unpublished crate version
- [ ] Replace placeholder JSON in CLI inspector with integration to `langgraph-inspector`

## Known Issues & Limitations 🐛
- [ ] PyO3 requires Python 3.8+
- [ ] Conditional edges need special handling
- [ ] Streaming not yet supported
- [ ] Human-in-the-loop needs design

## Research & Design Questions 🤔
- How to handle dynamic Python code?
- Best approach for tool abstraction?
- Should we support custom nodes?
- How to handle LangGraph updates?
- Versioning strategy for generated code?

## Success Metrics 📊
- [ ] Convert 10+ real-world LangGraph examples
- [ ] Achieve 5x+ performance improvement
- [ ] 90%+ test coverage
- [ ] < 100ms cold start time
- [ ] Support 95% of LangGraph features

## Dependencies & Prerequisites 📦
- Rust 1.75+
- Python 3.8+
- PyO3 0.22+
- LangGraph 0.2+
- Tokio 1.40+

## Timeline Estimate 🗓️
- Phase 1: ✅ Complete
- Phase 2: 2-3 weeks
- Phase 3: 3-4 weeks
- Phase 4: 2-3 weeks
- Phase 5: 2 weeks
- Phase 6: 3-4 weeks
- Phase 7: Ongoing

Total MVP: ~3 months
Production Ready: ~5 months

## How to Contribute 🤝
1. Pick an item from Phase 2 or Quick Wins
2. Create an issue to track progress
3. Submit PR with tests
4. Update this TODO when complete

## Contact & Support 💬
- GitHub Issues: [langgraph-rs/issues](https://github.com/genai-rs/langgraph-rs/issues)
- Discord: (To be created)
- Email: tim.van.wassenhove@gmail.com

## Reference Materials 📚
- [LangGraph GitHub repository](https://github.com/langchain-ai/langgraph)
- [Why LangGraph? concept guide](https://langchain-ai.github.io/langgraph/concepts/why-langgraph/)
- [PyO3 GitHub repository](https://github.com/PyO3/pyo3)
- [PyO3 getting started guide](https://pyo3.rs/main/getting-started.html)
