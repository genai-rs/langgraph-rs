"""Subprocess protocol adapter for loading one user graph and extracting metadata."""

from __future__ import annotations

import importlib.util
import json
import os
import sys
from pathlib import Path
from types import ModuleType
from typing import Any

from . import extract_graph_info

PROTOCOL_VERSION = 1
REQUEST_FIELDS = {"protocol_version", "operation", "source_file", "graph_object"}
SYNTHETIC_MODULE_NAME = "langgraph_cli_user_graph"


class ProtocolError(ValueError):
    """An invalid protocol request or an unsupported graph input."""


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ProtocolError(f"duplicate JSON field: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> None:
    raise ProtocolError(f"invalid JSON constant: {value}")


def _decode_json(text: str) -> Any:
    return json.loads(
        text,
        object_pairs_hook=_reject_duplicate_keys,
        parse_constant=_reject_constant,
    )


def _parse_request(text: str) -> dict[str, Any]:
    try:
        request = _decode_json(text)
    except (json.JSONDecodeError, ProtocolError) as exc:
        raise ProtocolError(f"invalid request JSON: {exc}") from exc
    if not isinstance(request, dict):
        raise ProtocolError("request must be a JSON object")
    if set(request) != REQUEST_FIELDS:
        missing = sorted(REQUEST_FIELDS - set(request))
        unknown = sorted(set(request) - REQUEST_FIELDS)
        details = []
        if missing:
            details.append(f"missing fields: {', '.join(missing)}")
        if unknown:
            details.append(f"unknown fields: {', '.join(unknown)}")
        raise ProtocolError("invalid request fields (" + "; ".join(details) + ")")
    if type(request["protocol_version"]) is not int:
        raise ProtocolError("protocol_version must be an integer")
    if request["protocol_version"] != PROTOCOL_VERSION:
        raise ProtocolError(f"unsupported protocol version: {request['protocol_version']}")
    if request["operation"] != "extract_graph_info":
        raise ProtocolError(f"unsupported operation: {request['operation']!r}")
    if not isinstance(request["source_file"], str) or not request["source_file"]:
        raise ProtocolError("source_file must be a non-empty string")
    if not isinstance(request["graph_object"], str) or not request["graph_object"]:
        raise ProtocolError("graph_object must be a non-empty string")
    selector = request["graph_object"].split(".")
    if any(not component.isidentifier() for component in selector):
        raise ProtocolError("graph_object must be a dotted sequence of Python identifiers")
    return request


def _load_graph(source_file: str, graph_object: str) -> Any:
    if not Path(source_file).is_absolute():
        raise ProtocolError("source_file must be an absolute path")
    try:
        path = Path(source_file).resolve(strict=True)
    except OSError as exc:
        raise ProtocolError(f"cannot resolve source_file {source_file!r}: {exc}") from exc
    if not path.is_file():
        raise ProtocolError(f"source_file is not a regular file: {path}")

    spec = importlib.util.spec_from_file_location(SYNTHETIC_MODULE_NAME, path)
    if spec is None or spec.loader is None:
        raise ProtocolError(f"cannot create an import loader for {path}")
    module: ModuleType = importlib.util.module_from_spec(spec)
    sys.modules[SYNTHETIC_MODULE_NAME] = module

    original_path = sys.path[:]
    sys.path.insert(0, str(path.parent))
    try:
        spec.loader.exec_module(module)
    except BaseException:
        sys.modules.pop(SYNTHETIC_MODULE_NAME, None)
        raise
    finally:
        sys.path[:] = original_path

    selected: Any = module.__dict__
    for index, component in enumerate(graph_object.split(".")):
        try:
            selected = selected[component] if index == 0 else getattr(selected, component)
        except (KeyError, AttributeError) as exc:
            raise ProtocolError(f"graph object {graph_object!r} was not found") from exc
    return selected


def _protocol_stdout() -> Any:
    """Save the protocol pipe, then redirect all ordinary stdout writes to stderr."""
    sys.stdout.flush()
    sys.stderr.flush()
    protocol_fd = os.dup(1)
    os.set_inheritable(protocol_fd, False)
    os.dup2(2, 1)
    return os.fdopen(protocol_fd, "w", encoding="utf-8", buffering=1)


def _diagnostic(message: str) -> None:
    single_line = " ".join(message.splitlines()).strip()
    print(f"langgraph-inspector: {single_line}", file=sys.stderr, flush=True)


def main() -> int:
    protocol = _protocol_stdout()
    try:
        request = _parse_request(sys.stdin.read())
        graph = _load_graph(request["source_file"], request["graph_object"])
        graph_info = _decode_json(extract_graph_info(graph))
        if not isinstance(graph_info, dict):
            raise ProtocolError("extractor returned a non-object GraphInfo value")
        response = {"protocol_version": PROTOCOL_VERSION, "graph_info": graph_info}
        output = json.dumps(response, ensure_ascii=False, allow_nan=False, separators=(",", ":"))
    except Exception as exc:
        _diagnostic(f"{type(exc).__name__}: {exc}")
        return 2
    try:
        protocol.write(output)
        protocol.write("\n")
        protocol.flush()
    except OSError as exc:
        _diagnostic(f"cannot write protocol response: {exc}")
        return 3
    finally:
        protocol.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
