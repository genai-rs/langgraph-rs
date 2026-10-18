
from types import SimpleNamespace

def start_node(state):
    return state


graph = SimpleNamespace(
    nodes={"start": SimpleNamespace(func=start_node)},
    edges=[("start", "__end__")],
    channels={"count": SimpleNamespace(value_type=int)},
    entry_point="start",
)
