# Contributing to langgraph-rs

Thank you for your interest in contributing to langgraph-rs! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and collaborative environment.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Python 3.8 or later
- Git

### Setting Up Your Development Environment

1. **Fork and clone the repository:**
   ```bash
   git fork https://github.com/genai-rs/langgraph-rs
   cd langgraph-rs
   ```

2. **Set up Python environment:**
   ```bash
   ./scripts/setup-python.sh  # Unix/Mac
   # or
   .\scripts\setup-python.ps1 # Windows
   ```

3. **Build the project:**
   ```bash
   cargo build --workspace
   ```

4. **Run tests:**
   ```bash
   cargo test --workspace
   ```

## Development Workflow

### Branch Strategy

We use a **feature branch** workflow:

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   # or for exploration:
   git checkout -b explore/what-you-are-exploring
   ```

2. **Make your changes** and commit frequently
3. **Push your branch** and create a Pull Request
4. **Never push directly to main**

### Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `test:` - Adding or updating tests
- `build:` - Build system changes
- `ci:` - CI/CD changes
- `chore:` - Other changes

**Examples:**
```bash
git commit -m "feat: add PyO3 introspection for node extraction"
git commit -m "fix: handle None values in Python type mapping"
git commit -m "docs: update README with installation instructions"
```

**Breaking Changes:**
```bash
git commit -m "feat!: change API for graph execution

BREAKING CHANGE: execute() now returns Result<GraphState> instead of GraphState"
```

### Code Quality Standards

Before submitting a PR, ensure your code meets these standards:

1. **Format your code:**
   ```bash
   cargo fmt --all
   ```

2. **Run clippy:**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Ensure tests pass:**
   ```bash
   cargo test --workspace
   ```

4. **Python code quality** (for test fixtures):
   ```bash
   black tests/fixtures/
   ruff check tests/fixtures/
   mypy tests/fixtures/
   ```

### Testing Guidelines

- Write unit tests for new functionality
- Add integration tests for major features
- Ensure test coverage stays above 90%
- Test edge cases and error conditions
- Use descriptive test names

Example:
```rust
#[test]
fn test_map_python_list_type_to_rust_vec() {
    let result = map_python_type("list[str]");
    assert_eq!(result, RustType::Vec(Box::new(RustType::String)));
}
```

### Documentation

- Add doc comments for public APIs
- Update README.md for user-facing changes
- Update docs/ for architectural changes
- Include examples in doc comments

Example:
```rust
/// Maps a Python type string to the corresponding Rust type.
///
/// # Examples
///
/// ```
/// use langgraph_inspector::type_mapping::map_python_type;
///
/// let rust_type = map_python_type("list[str]");
/// assert_eq!(rust_type.to_rust_string(), "Vec<String>");
/// ```
pub fn map_python_type(py_type: &str) -> RustType {
    // Implementation
}
```

## Pull Request Process

1. **Update documentation** if needed
2. **Add tests** for new functionality
3. **Ensure CI passes** - all checks must be green
4. **Request review** from maintainers
5. **Address review comments**
6. **Squash commits** if requested

### PR Title Format

Use conventional commit format for PR titles:
- `feat: add support for tool nodes`
- `fix: handle edge case in type mapping`
- `docs: improve installation guide`

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings introduced
- [ ] Tests pass locally
```

## Project Structure

```
langgraph-rs/
├── langgraph-inspector/   # Python introspection via PyO3
├── langgraph-generator/   # Rust code generation
├── langgraph-runtime/     # Runtime support library
├── langgraph-cli/         # Command-line interface
├── tests/                 # Integration tests
├── docs/                  # Documentation
└── examples/              # Example workflows
```

## Areas to Contribute

Check our [TODO.md](TODO.md) for current priorities. Good areas for contribution:

- **Quick Wins**: Small, well-defined tasks
- **Documentation**: Improve guides and examples
- **Testing**: Add test coverage
- **Bug Fixes**: Check GitHub issues
- **Features**: Implement items from Phase 2-7

## Debugging Tips

### Enable logging:
```bash
RUST_LOG=debug cargo run --bin langgraph-rs -- inspect examples/simple_workflow.py
```

### Run specific tests:
```bash
cargo test -p langgraph-inspector test_type_mapping
```

### Check Python linking issues:
```bash
python3-config --ldflags
```

## Common Issues

### PyO3 Linking Errors

If you get Python linking errors:
```bash
# Check Python version
python3 --version

# Set Python path manually
export PYO3_PYTHON=/usr/bin/python3
cargo build
```

### Clippy Warnings

Address all clippy warnings. If you must suppress one, add a comment explaining why:
```rust
#[allow(clippy::too_many_arguments)]
// This function legitimately needs all these parameters
fn complex_function(...) {
}
```

## Getting Help

- **GitHub Issues**: [langgraph-rs/issues](https://github.com/genai-rs/langgraph-rs/issues)
- **Discussions**: GitHub Discussions (coming soon)
- **Email**: tim.van.wassenhove@gmail.com

## Recognition

Contributors will be:
- Listed in our Contributors section
- Mentioned in release notes
- Credited in generated code comments (optionally)

## License

By contributing, you agree that your contributions will be dual-licensed under MIT or Apache-2.0, matching the project license.

---

Thank you for contributing to langgraph-rs! 🚀
