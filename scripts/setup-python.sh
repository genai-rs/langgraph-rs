#!/usr/bin/env bash
# Setup Python virtual environment for langgraph-rs development using uv

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Setting up Python environment for langgraph-rs using uv..."

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "uv is not installed. Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    echo "Please restart your shell or run: source $HOME/.cargo/env"
    exit 1
fi

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo "Detected Python version: $PYTHON_VERSION"

# Create or sync virtual environment with uv
echo "Syncing Python environment with uv..."
cd "$PROJECT_DIR"
uv sync --all-extras

echo ""
echo "Python environment setup complete!"
echo ""
echo "To activate the virtual environment, run:"
echo "  source .venv/bin/activate"
echo ""
echo "Or use uv run to execute commands:"
echo "  uv run python <script.py>"
echo "  uv run pytest"
echo ""
echo "To add new dependencies:"
echo "  uv add <package>"
echo ""
echo "To deactivate, run:"
echo "  deactivate"
