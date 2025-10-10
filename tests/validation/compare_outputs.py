#!/usr/bin/env python3
"""
Compare outputs between Python LangGraph and generated Rust code.
This validates that the Rust implementation produces equivalent results.
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict


def run_python_workflow(workflow_path: Path) -> Dict[str, Any]:
    """Execute Python LangGraph workflow and return final state."""
    # Import and run the workflow
    import importlib.util

    spec = importlib.util.spec_from_file_location("workflow", workflow_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    # Get the compiled graph
    graph = module.graph

    # Run with initial state
    initial_state = getattr(module, "initial_state", {})
    result = graph.invoke(initial_state)

    return result


def run_rust_workflow(rust_binary: Path, input_state: Dict[str, Any]) -> Dict[str, Any]:
    """Execute generated Rust workflow and return final state."""
    # Write input state to JSON
    input_json = json.dumps(input_state)

    # Run Rust binary
    result = subprocess.run(
        [rust_binary],
        input=input_json,
        capture_output=True,
        text=True,
        check=True
    )

    # Parse output
    return json.loads(result.stdout)


def compare_states(python_state: Dict, rust_state: Dict, tolerance: float = 1e-6) -> bool:
    """Compare Python and Rust states for equivalence."""
    if set(python_state.keys()) != set(rust_state.keys()):
        print(f"Key mismatch: Python={python_state.keys()}, Rust={rust_state.keys()}")
        return False

    for key in python_state:
        py_val = python_state[key]
        rs_val = rust_state[key]

        # Handle different types
        if isinstance(py_val, (int, float)) and isinstance(rs_val, (int, float)):
            if abs(py_val - rs_val) > tolerance:
                print(f"Numeric mismatch for '{key}': {py_val} != {rs_val}")
                return False
        elif isinstance(py_val, list) and isinstance(rs_val, list):
            if len(py_val) != len(rs_val):
                print(f"List length mismatch for '{key}': {len(py_val)} != {len(rs_val)}")
                return False
            for i, (pv, rv) in enumerate(zip(py_val, rs_val)):
                if pv != rv:
                    print(f"List element mismatch for '{key}'[{i}]: {pv} != {rv}")
                    return False
        elif py_val != rs_val:
            print(f"Value mismatch for '{key}': {py_val} != {rs_val}")
            return False

    return True


def main():
    """Main validation routine."""
    if len(sys.argv) < 2:
        print("Usage: compare_outputs.py <workflow_name>")
        sys.exit(1)

    workflow_name = sys.argv[1]
    project_root = Path(__file__).parent.parent.parent

    python_workflow = project_root / "tests" / "fixtures" / f"{workflow_name}.py"
    rust_binary = project_root / "target" / "debug" / f"{workflow_name}"

    if not python_workflow.exists():
        print(f"Python workflow not found: {python_workflow}")
        sys.exit(1)

    print(f"Validating {workflow_name}...")
    print("=" * 60)

    try:
        # Run Python version
        print("Running Python workflow...")
        python_result = run_python_workflow(python_workflow)
        print(f"Python result: {python_result}")

        # Run Rust version
        if rust_binary.exists():
            print("\nRunning Rust workflow...")
            rust_result = run_rust_workflow(rust_binary, python_result)
            print(f"Rust result: {rust_result}")

            # Compare results
            print("\nComparing results...")
            if compare_states(python_result, rust_result):
                print("✅ Results match!")
                return 0
            else:
                print("❌ Results differ!")
                return 1
        else:
            print(f"\n⚠️  Rust binary not found: {rust_binary}")
            print("Skipping Rust comparison")
            return 0

    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
