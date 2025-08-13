# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services memory module."""

from __future__ import annotations

import contextlib
import json
from pathlib import Path
import threading
from typing import Any

import pytest

from bijux_cli.contracts import MemoryProtocol
from bijux_cli.services.memory import Memory


@pytest.fixture
def memory_file(tmp_path: Path) -> Path:
    """Provide a temporary file path for the memory store."""
    return tmp_path / "memory.json"


@pytest.fixture
def memory(monkeypatch: pytest.MonkeyPatch, memory_file: Path) -> Memory:
    """Provide a Memory instance with a patched file path."""
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", memory_file)
    return Memory()


def test_init_no_file(memory: Memory) -> None:
    """Test that initialization with no existing file results in an empty store."""
    assert not memory.keys()


def test_init_with_valid_file(
    monkeypatch: pytest.MonkeyPatch, memory_file: Path
) -> None:
    """Test that initialization with a valid JSON file loads the data."""
    memory_file.write_text('{"key": "value"}')
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", memory_file)
    m = Memory()
    assert m.get("key") == "value"


def test_init_with_invalid_json(
    monkeypatch: pytest.MonkeyPatch, memory_file: Path
) -> None:
    """Test that initialization with invalid JSON results in an empty store."""
    memory_file.write_text("invalid json")
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", memory_file)
    m = Memory()
    assert not m.keys()


def test_init_with_non_dict_json(
    monkeypatch: pytest.MonkeyPatch, memory_file: Path
) -> None:
    """Test that initialization with a non-dictionary JSON object raises an error."""
    memory_file.write_text("[1,2,3]")
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", memory_file)
    m = Memory()
    with pytest.raises(AttributeError):
        m.keys()


@pytest.mark.parametrize(
    ("key", "value"), [("a", 1), ("b", "str"), ("c", [1, 2]), ("d", {"e": 3})]
)
def test_set_and_get(memory: Memory, key: str, value: Any) -> None:
    """Test setting and getting various JSON-serializable types."""
    memory.set(key, value)
    assert memory.get(key) == value


def test_get_nonexistent(memory: Memory) -> None:
    """Test that getting a non-existent key raises a KeyError."""
    with pytest.raises(KeyError, match="Memory key not found: nonexistent"):
        memory.get("nonexistent")


def test_set_overwrite(memory: Memory) -> None:
    """Test that setting an existing key overwrites its value."""
    memory.set("key", "old")
    memory.set("key", "new")
    assert memory.get("key") == "new"


def test_delete_existing(memory: Memory) -> None:
    """Test that deleting an existing key removes it from the store."""
    memory.set("key", "value")
    memory.delete("key")
    with pytest.raises(KeyError):
        memory.get("key")


def test_delete_nonexistent(memory: Memory) -> None:
    """Test that deleting a non-existent key raises a KeyError."""
    with pytest.raises(KeyError, match="Memory key not found: nonexistent"):
        memory.delete("nonexistent")


def test_clear_empty(memory: Memory) -> None:
    """Test that clearing an already empty store has no effect."""
    memory.clear()
    assert not memory.keys()


def test_clear_non_empty(memory: Memory) -> None:
    """Test that clearing a non-empty store removes all keys."""
    memory.set("a", 1)
    memory.set("b", 2)
    memory.clear()
    assert not memory.keys()


def test_keys_empty(memory: Memory) -> None:
    """Test that keys() on an empty store returns an empty list."""
    assert not memory.keys()


def test_keys_non_empty(memory: Memory) -> None:
    """Test that keys() on a non-empty store returns all keys."""
    memory.set("a", 1)
    memory.set("b", 2)
    assert sorted(memory.keys()) == ["a", "b"]


def test_persist_after_set(memory: Memory, memory_file: Path) -> None:
    """Test that a 'set' operation is persisted to the file."""
    memory.set("key", "value")
    assert json.loads(memory_file.read_text()) == {"key": "value"}


def test_persist_after_delete(memory: Memory, memory_file: Path) -> None:
    """Test that a 'delete' operation is persisted to the file."""
    memory.set("key", "value")
    memory.delete("key")
    assert not json.loads(memory_file.read_text())


def test_persist_after_clear(memory: Memory, memory_file: Path) -> None:
    """Test that a 'clear' operation is persisted to the file."""
    memory.set("key", "value")
    memory.clear()
    assert not json.loads(memory_file.read_text())


def test_set_non_serializable(memory: Memory) -> None:
    """Test that setting a non-JSON-serializable value raises a TypeError."""
    with pytest.raises(TypeError):
        memory.set("key", lambda: None)


@pytest.mark.parametrize(("num_threads", "num_sets"), [(2, 100), (5, 200), (10, 50)])
def test_thread_safety_set(memory: Memory, num_threads: int, num_sets: int) -> None:
    """Test thread safety with multiple concurrent 'set' operations."""

    def worker() -> None:
        for i in range(num_sets):
            memory.set(f"key_{threading.current_thread().ident}_{i}", i)

    threads = [threading.Thread(target=worker) for _ in range(num_threads)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert len(memory.keys()) == num_threads * num_sets


@pytest.mark.parametrize(("num_threads", "num_ops"), [(3, 50), (4, 100)])
def test_thread_safety_mixed_ops(
    memory: Memory, num_threads: int, num_ops: int
) -> None:
    """Test thread safety with a mix of concurrent read and write operations."""

    def worker() -> None:
        for i in range(num_ops):
            key = f"key_{i}"
            memory.set(key, i)
            if i % 2 == 0:
                with contextlib.suppress(KeyError):
                    memory.get(key)
            if i % 3 == 0:
                with contextlib.suppress(KeyError):
                    memory.delete(key)

    threads = [threading.Thread(target=worker) for _ in range(num_threads)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()


@pytest.mark.parametrize("key", [f"key_{i}" for i in range(20)])
def test_multiple_sets(memory: Memory, key: str) -> None:
    """Test setting multiple different keys."""
    memory.set(key, key)
    assert memory.get(key) == key


@pytest.mark.parametrize("key", [f"key_{i}" for i in range(20)])
def test_multiple_deletes_after_set(memory: Memory, key: str) -> None:
    """Test deleting multiple keys that were previously set."""
    memory.set(key, key)
    memory.delete(key)
    with pytest.raises(KeyError):
        memory.get(key)


def test_clear_multiple_times(memory: Memory) -> None:
    """Test that clearing the store multiple times works correctly."""
    for _ in range(5):
        memory.set("key", "value")
        memory.clear()
    assert not memory.keys()


def test_keys_after_operations(memory: Memory) -> None:
    """Test that keys() reflects a series of mixed operations."""
    memory.set("a", 1)
    memory.set("b", 2)
    memory.delete("a")
    memory.set("c", 3)
    assert sorted(memory.keys()) == ["b", "c"]


def test_large_number_of_keys(memory: Memory) -> None:
    """Test the memory store's handling of a large number of keys."""
    for i in range(100):
        memory.set(f"key_{i}", i)
    assert len(memory.keys()) == 100


def test_concurrent_get_set(memory: Memory) -> None:
    """Test concurrent get and set operations with unique keys per thread."""

    def setter(tid: int) -> None:
        for i in range(100):
            key = f"key_{tid}_{i}"
            memory.set(key, i)
            assert memory.get(key) == i

    threads = [threading.Thread(target=setter, args=(i,)) for i in range(10)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert len(memory.keys()) == 1000


@pytest.mark.parametrize(
    "value", [None, True, False, 0, 1.5, "", [], {}, {"nested": [1, 2]}]
)
def test_set_get_different_types(memory: Memory, value: Any) -> None:
    """Test setting and getting a variety of JSON-serializable types."""
    memory.set("key", value)
    assert memory.get("key") == value


def test_set_after_clear(memory: Memory) -> None:
    """Test that a 'set' operation works correctly after a 'clear'."""
    memory.set("old", "value")
    memory.clear()
    memory.set("new", "value")
    assert memory.get("new") == "value"
    with pytest.raises(KeyError):
        memory.get("old")


def test_delete_after_clear(memory: Memory) -> None:
    """Test that deleting a key after a 'clear' raises a KeyError."""
    memory.set("key", "value")
    memory.clear()
    with pytest.raises(KeyError):
        memory.delete("key")


def test_persist_with_empty_dict_after_clear(memory: Memory, memory_file: Path) -> None:
    """Test that the persisted file is an empty dictionary after a 'clear'."""
    memory.clear()
    assert not json.loads(memory_file.read_text())


def test_persist_with_multiple_keys(memory: Memory, memory_file: Path) -> None:
    """Test that multiple keys are correctly persisted to the file."""
    memory.set("a", 1)
    memory.set("b", 2)
    assert json.loads(memory_file.read_text()) == {"a": 1, "b": 2}


def test_init_with_empty_file(
    monkeypatch: pytest.MonkeyPatch, memory_file: Path
) -> None:
    """Test that initializing from an empty JSON file results in an empty store."""
    memory_file.write_text("{}")
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", memory_file)
    m = Memory()
    assert not m.keys()


def test_init_with_file_permission_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a PermissionError during file read on init is propagated."""

    def mock_open(self: Path, *args: Any, **kwargs: Any) -> None:
        raise PermissionError

    monkeypatch.setattr(Path, "open", mock_open)
    with pytest.raises(PermissionError):
        Memory()


def test_persist_permission_error(
    memory: Memory, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that a PermissionError during file write on persist is propagated."""

    def mock_open(self: Path, *args: Any, **kwargs: Any) -> None:
        if "w" in args[0]:
            raise PermissionError

    monkeypatch.setattr(Path, "open", mock_open)
    with pytest.raises(PermissionError):
        memory.set("key", "value")


@pytest.mark.parametrize("num_clears", [1, 5, 10])
def test_multiple_clears(memory: Memory, num_clears: int) -> None:
    """Test that the store can be cleared multiple times."""
    for _ in range(num_clears):
        memory.set("key", "value")
        memory.clear()
    assert not memory.keys()


@pytest.mark.parametrize("keys", [["a", "b"], ["x"], list("abcde")])
def test_keys_with_various_lengths(memory: Memory, keys: list[str]) -> None:
    """Test the keys() method with a varying number of entries in the store."""
    for k in keys:
        memory.set(k, k)
    assert sorted(memory.keys()) == sorted(keys)


def test_set_with_complex_key(memory: Memory) -> None:
    """Test setting a key that contains special characters like slashes."""
    memory.set("key/with/slash", "value")
    assert memory.get("key/with/slash") == "value"


def test_set_with_unicode_key(memory: Memory) -> None:
    """Test setting a key that contains Unicode characters."""
    memory.set("clé", "value")
    assert memory.get("clé") == "value"


def test_set_with_unicode_value(memory: Memory) -> None:
    """Test setting a value that contains Unicode characters."""
    memory.set("key", "valeur")
    assert memory.get("key") == "valeur"


def test_large_value(memory: Memory) -> None:
    """Test setting and getting a very large string value."""
    large_val = "x" * 1000000
    memory.set("large", large_val)
    assert memory.get("large") == large_val


def test_many_operations(memory: Memory) -> None:
    """Test a long sequence of mixed set and delete operations."""
    for i in range(100):
        memory.set(f"key{i}", i)
        if i % 2 == 0:
            memory.delete(f"key{i}")
    assert len(memory.keys()) == 50


def test_thread_safety_delete(memory: Memory) -> None:
    """Test thread safety with multiple concurrent 'delete' operations."""
    for i in range(100):
        memory.set(f"key{i}", i)

    def worker() -> None:
        for i in range(100):
            if i % 2 == 0:
                with contextlib.suppress(KeyError):
                    memory.delete(f"key{i}")

    threads = [threading.Thread(target=worker) for _ in range(5)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    remaining = len(memory.keys())
    assert remaining <= 50


def test_thread_safety_clear(memory: Memory) -> None:
    """Test thread safety with concurrent 'set' and 'clear' operations."""

    def setter() -> None:
        for i in range(10):
            memory.set(f"key{i}", i)

    def clearer() -> None:
        memory.clear()

    threads = [threading.Thread(target=setter) for _ in range(5)] + [
        threading.Thread(target=clearer) for _ in range(2)
    ]
    for t in threads:
        t.start()
    for t in threads:
        t.join()


@pytest.mark.parametrize("op", ["set", "delete", "clear", "get", "keys"])
def test_operations_on_empty(memory: Memory, op: str) -> None:
    """Test various operations on an initially empty store."""
    if op == "set":
        memory.set("key", "value")
        assert memory.get("key") == "value"
    elif op == "delete":
        with pytest.raises(KeyError):
            memory.delete("key")
    elif op == "clear":
        memory.clear()
        assert not memory.keys()
    elif op == "get":
        with pytest.raises(KeyError):
            memory.get("key")
    elif op == "keys":
        assert not memory.keys()


def test_set_get_delete_cycle(memory: Memory) -> None:
    """Test a repeated cycle of set, get, and delete on the same key."""
    for i in range(10):
        memory.set("key", i)
        assert memory.get("key") == i
        memory.delete("key")
        with pytest.raises(KeyError):
            memory.get("key")


def test_persist_after_multiple_sets(memory: Memory, memory_file: Path) -> None:
    """Test that the file state is correct after multiple 'set' operations."""
    for i in range(5):
        memory.set(f"key{i}", i)
    assert json.loads(memory_file.read_text()) == {f"key{i}": i for i in range(5)}


def test_persist_after_mixed_ops(memory: Memory, memory_file: Path) -> None:
    """Test that the file state is correct after a mix of operations."""
    memory.set("a", 1)
    memory.set("b", 2)
    memory.delete("a")
    memory.set("c", 3)
    assert json.loads(memory_file.read_text()) == {"b": 2, "c": 3}


def test_init_with_large_file(
    monkeypatch: pytest.MonkeyPatch, memory_file: Path
) -> None:
    """Test initialization from a large data file."""
    large_data = {f"key{i}": i for i in range(1000)}
    memory_file.write_text(json.dumps(large_data))
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", memory_file)
    m = Memory()
    assert len(m.keys()) == 1000


def test_keys_type(memory: Memory) -> None:
    """Test that the keys() method returns a list of strings."""
    memory.set("a", 1)
    keys = memory.keys()
    assert isinstance(keys, list)
    assert all(isinstance(k, str) for k in keys)


def test_memory_protocol_conformance(memory: Memory) -> None:
    """Test that the Memory class conforms to the MemoryProtocol."""
    assert isinstance(memory, MemoryProtocol)


@pytest.mark.parametrize("invalid_value", [object(), set(), lambda: None])
def test_set_invalid_json_types(memory: Memory, invalid_value: Any) -> None:
    """Test that setting a non-JSON-serializable value raises a TypeError."""
    with pytest.raises(TypeError):
        memory.set("key", invalid_value)


def test_concurrent_keys_calls(memory: Memory) -> None:
    """Test that calling keys() concurrently from multiple threads does not crash."""
    memory.set("a", 1)

    def caller() -> None:
        for _ in range(100):
            memory.keys()

    threads = [threading.Thread(target=caller) for _ in range(10)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()


def test_set_during_keys(memory: Memory) -> None:
    """Test a concurrent 'set' operation during a 'keys' call."""
    barrier = threading.Barrier(2)

    def key_caller() -> None:
        barrier.wait()
        memory.keys()

    def setter() -> None:
        barrier.wait()
        memory.set("new", "value")

    t1 = threading.Thread(target=key_caller)
    t2 = threading.Thread(target=setter)
    t1.start()
    t2.start()
    t1.join()
    t2.join()


@pytest.mark.parametrize("i", range(10))
def test_repeated_set_same_key(memory: Memory, i: int) -> None:
    """Test that repeatedly setting the same key updates its value."""
    memory.set("key", i)
    assert memory.get("key") == i


@pytest.mark.parametrize("i", range(10))
def test_repeated_delete_nonexistent(memory: Memory, i: int) -> None:
    """Test that repeatedly deleting a non-existent key consistently raises KeyError."""
    with pytest.raises(KeyError):
        memory.delete(f"non{i}")


def test_clear_then_keys(memory: Memory) -> None:
    """Test that keys() returns an empty list immediately after a 'clear'."""
    memory.clear()
    assert not memory.keys()


def test_set_then_clear_then_set(memory: Memory) -> None:
    """Test a set-clear-set sequence to ensure the store is in the correct final state."""
    memory.set("a", 1)
    memory.clear()
    memory.set("b", 2)
    assert memory.get("b") == 2


def test_delete_all_keys(memory: Memory) -> None:
    """Test that deleting all keys one by one results in an empty store."""
    for i in range(5):
        memory.set(f"key{i}", i)
    for i in range(5):
        memory.delete(f"key{i}")
    assert not memory.keys()


def test_memory_file_creation(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Test that the memory file is created on the first persistence operation."""
    mem_file = tmp_path / "mem.json"
    assert not mem_file.exists()
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", mem_file)
    m = Memory()
    m.set("key", "value")
    assert mem_file.exists()


def test_parent_dir_creation(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Test that the parent directory for the memory file is created if it doesn't exist."""
    mem_file = tmp_path / "subdir" / "mem.json"
    monkeypatch.setattr("bijux_cli.services.memory.MEMORY_FILE", mem_file)
    m = Memory()
    m.set("key", "value")
    assert mem_file.exists()


@pytest.mark.parametrize(("key", "value"), [(f"k{i}", i) for i in range(20)])
def test_param_set_get(memory: Memory, key: str, value: int) -> None:
    """Test a parametrized sequence of set and get operations."""
    memory.set(key, value)
    assert memory.get(key) == value


@pytest.mark.parametrize("key", [f"k{i}" for i in range(20)])
def test_param_delete_after_set(memory: Memory, key: str) -> None:
    """Test a parametrized sequence of set and delete operations."""
    memory.set(key, "value")
    memory.delete(key)
    with pytest.raises(KeyError):
        memory.get(key)
