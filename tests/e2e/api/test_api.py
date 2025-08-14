# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""E2E tests for the Bijux CLI HTTP API."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Any
import uuid


def _detail_contains(err: dict[str, Any], needle: str) -> bool:
    """Return True if the FastAPI 'detail' payload mentions `needle` somewhere."""
    detail = err.get("detail")
    if isinstance(detail, str):
        return needle.lower() in detail.lower()
    if isinstance(detail, Iterable):
        for entry in detail:
            if isinstance(entry, dict):
                loc = entry.get("loc")
                if loc and needle.lower() in "/".join(map(str, loc)).lower():
                    return True
                msg = entry.get("msg")
                if isinstance(msg, str) and needle.lower() in msg.lower():
                    return True
            else:
                if needle.lower() in str(entry).lower():
                    return True
    return False


def test_list_items_default(client: Any) -> None:
    """Test listing items with default pagination."""
    resp = client.get("/items")
    assert resp.status_code == 200
    body = resp.json()
    assert isinstance(body, dict)
    assert "items" in body
    assert isinstance(body["items"], list)
    assert "total" in body
    assert isinstance(body["total"], int)


def test_list_items_with_pagination(client: Any) -> None:
    """Test listing items with a custom limit and offset."""
    resp = client.get("/items", params={"limit": 5, "offset": 0})
    assert resp.status_code == 200
    body = resp.json()
    assert len(body["items"]) <= 5
    assert body["total"] >= 0


def test_list_items_invalid_limit(client: Any) -> None:
    """Test that a negative limit results in a validation error."""
    resp = client.get("/items", params={"limit": -1})
    assert resp.status_code == 422
    err = resp.json()
    assert "detail" in err
    assert _detail_contains(err, "limit")


def test_list_items_large_limit(client: Any) -> None:
    """Test that a large limit is handled without crashing."""
    resp = client.get("/items", params={"limit": 1000})
    assert resp.status_code in (200, 422)
    if resp.status_code == 200:
        assert len(resp.json()["items"]) <= 1000


def test_list_items_invalid_offset(client: Any) -> None:
    """Test that a negative offset results in a validation error."""
    resp = client.get("/items", params={"offset": -10})
    assert resp.status_code == 422
    err = resp.json()
    assert "detail" in err
    assert _detail_contains(err, "offset")


def test_get_item_by_id_valid(client: Any, create_test_item: int) -> None:
    """Test retrieving an existing item by its ID."""
    item_id = create_test_item
    resp = client.get(f"/items/{item_id}")
    assert resp.status_code == 200
    item = resp.json()
    assert "id" in item
    assert item["id"] == item_id
    assert "name" in item


def test_get_item_by_id_not_found(client: Any) -> None:
    """Test that retrieving a non-existent item returns a 404 error."""
    resp = client.get("/items/999999")
    assert resp.status_code == 404
    err = resp.json()
    assert "detail" in err
    assert "not found" in err["detail"].lower()


def test_get_item_by_invalid_id_format(client: Any) -> None:
    """Test that a non-integer item ID results in a validation error."""
    resp = client.get("/items/abc")
    assert resp.status_code == 422
    err = resp.json()
    assert "detail" in err
    assert _detail_contains(err, "path")


def test_get_item_by_edge_id_zero(client: Any) -> None:
    """Test that an item ID of zero results in a validation error."""
    resp = client.get("/items/0")
    assert resp.status_code == 422


def test_get_item_by_negative_id(client: Any) -> None:
    """Test that a negative item ID results in a validation error."""
    resp = client.get("/items/-1")
    assert resp.status_code == 422


def test_create_item_valid(client: Any) -> None:
    """Test creating a new item with valid data."""
    payload = {"name": f"New Item {uuid.uuid4()}", "description": "Test creation"}
    resp = client.post("/items", json=payload)
    assert resp.status_code == 201
    item = resp.json()
    assert "id" in item
    assert item["name"].startswith("New Item ")
    client.delete(f"/items/{item['id']}")


def test_create_item_missing_field(client: Any) -> None:
    """Test that creating an item with a missing required field fails."""
    resp = client.post("/items", json={"description": "Missing name"})
    assert resp.status_code == 422
    err = resp.json()
    assert "detail" in err
    assert _detail_contains(err, "name")


def test_create_item_invalid_type(client: Any) -> None:
    """Test that creating an item with an invalid data type fails."""
    resp = client.post("/items", json={"name": 123, "description": "Invalid name type"})
    assert resp.status_code == 422
    err = resp.json()
    assert "detail" in err
    assert _detail_contains(err, "name")


def test_create_item_duplicate(client: Any, create_test_item: int) -> None:
    """Create once, then create again with the SAME name → 409."""
    base = f"Dup {uuid.uuid4()}"
    first = client.post("/items", json={"name": base, "description": "dup seed"})
    assert first.status_code == 201
    try:
        second = client.post("/items", json={"name": base, "description": "dup again"})
        assert second.status_code == 409
    finally:
        client.delete(f"/items/{first.json()['id']}")


def test_create_item_large_payload(client: Any) -> None:
    """Test that creating an item with a very large payload is handled."""
    large_desc = "A" * 10000
    unique_name = f"Large Item {uuid.uuid4()}"
    payload = {"name": unique_name, "description": large_desc}
    resp = client.post("/items", json=payload)
    assert resp.status_code in (201, 422)


def test_update_item_valid(client: Any, create_test_item: int) -> None:
    """Test updating an existing item with valid data."""
    item_id = create_test_item
    payload = {"name": "Updated Name", "description": "Updated desc"}
    resp = client.put(f"/items/{item_id}", json=payload)
    assert resp.status_code == 200
    updated = resp.json()
    assert updated["name"] == "Updated Name"


def test_update_item_not_found(client: Any) -> None:
    """Test that updating a non-existent item returns a 404 error."""
    payload = {"name": "Ghost", "description": "Doesn't exist"}
    resp = client.put("/items/999999", json=payload)
    assert resp.status_code == 404


def test_update_item_invalid_data(client: Any, create_test_item: int) -> None:
    """Test that updating an item with invalid data fails."""
    item_id = create_test_item
    payload = {"name": 123}
    resp = client.put(f"/items/{item_id}", json=payload)
    assert resp.status_code == 422


def test_delete_item_valid(client: Any, create_test_item: int) -> None:
    """Test deleting an existing item."""
    item_id = create_test_item
    resp = client.delete(f"/items/{item_id}")
    assert resp.status_code == 204
    get_resp = client.get(f"/items/{item_id}")
    assert get_resp.status_code == 404


def test_delete_item_not_found(client: Any) -> None:
    """Test that deleting a non-existent item is handled gracefully."""
    resp = client.delete("/items/999999")
    assert resp.status_code in (404, 204)
