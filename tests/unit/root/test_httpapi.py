# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root httpapi module."""

from __future__ import annotations

import ast

from fastapi.testclient import TestClient

import bijux_cli.httpapi as api

# pyright: reportPrivateUsage=false


def _client() -> TestClient:
    """Create a new test client for the FastAPI application."""
    return TestClient(api.app)


def test_runtime_checkable_protocol_and_dependency() -> None:
    """Test that the item store protocol is checkable and the dependency is correct."""
    assert api.get_store() is api.store
    assert isinstance(api.store, api.InMemoryItemStore)
    assert isinstance(api.store, api.ItemStoreProtocol)


def test_lifespan_prepopulates_and_shutdown_resets() -> None:
    """Test that the lifespan event handler correctly populates and clears the store."""
    with _client() as client:
        r = client.get("/v1/items")
        assert r.status_code == 200
        data = r.json()
        names = {it["name"] for it in data["items"]}
        assert names == {"Item One", "Item Two"}
        assert data["total"] == 2

    items, total = api.store.list_items(limit=100, offset=0)
    assert total == 0
    assert not items


def test_create_get_update_delete_flow_and_conflicts() -> None:
    """Test the full CRUD lifecycle of an item, including conflict handling."""
    with _client() as client:
        body = {"name": "  New  ", "description": "D"}
        r = client.post("/v1/items", json=body)
        assert r.status_code == 201
        created = r.json()
        assert created["name"] == "New"
        assert created["description"] == "D"
        created_id = created["id"]

        r = client.get("/v1/items")
        assert r.status_code == 200
        data = r.json()
        assert data["total"] == 3
        ids = [it["id"] for it in data["items"]]
        assert created_id in ids

        r = client.get(f"/v1/items/{created_id}")
        assert r.status_code == 200
        assert r.json()["id"] == created_id

        r = client.put(
            f"/v1/items/{created_id}", json={"name": "Item Two", "description": "X"}
        )
        assert r.status_code == 409
        assert r.json()["detail"] == "Item with this name already exists"

        r = client.put(
            f"/v1/items/{created_id}", json={"name": "Renamed", "description": "X"}
        )
        assert r.status_code == 200
        assert r.json()["name"] == "Renamed"

        r = client.delete(f"/v1/items/{created_id}")
        assert r.status_code == 204
        assert not r.text

        r = client.get(f"/v1/items/{created_id}")
        assert r.status_code == 404
        assert r.json()["detail"] == "Item not found"


def test_conflict_on_duplicate_create() -> None:
    """Test that creating an item with a duplicate name results in a 409 conflict."""
    with _client() as client:
        r = client.post("/v1/items", json={"name": "Item One", "description": "dup"})
        assert r.status_code == 409
        assert r.json()["detail"] == "Item with this name already exists"


def test_update_and_delete_nonexistent_404() -> None:
    """Test that updating or deleting a non-existent item results in a 404 error."""
    with _client() as client:
        r = client.put("/v1/items/999", json={"name": "X", "description": None})
        assert r.status_code == 404
        assert r.json()["detail"] == "Item not found"

        r = client.delete("/v1/items/999")
        assert r.status_code == 404
        assert r.json()["detail"] == "Item not found"


def test_get_nonexistent_404() -> None:
    """Test that getting a non-existent item results in a 404 error."""
    with _client() as client:
        r = client.get("/v1/items/999")
        assert r.status_code == 404
        assert r.json()["detail"] == "Item not found"


def test_validation_handler_422_for_body_and_path() -> None:
    """Test that validation errors for request bodies and path parameters result in a 422 error."""
    with _client() as client:
        r = client.post("/v1/items", json={"name": "", "description": "x"})
        assert r.status_code == 422
        detail = r.json().get("detail")
        errors = ast.literal_eval(detail)
        assert isinstance(errors, list)
        assert any(e.get("type") == "string_too_short" for e in errors)

        r = client.get("/v1/items/0")
        assert r.status_code == 422
        detail = r.json().get("detail")
        errors = ast.literal_eval(detail)
        assert isinstance(errors, list)
        assert any(e.get("type") == "greater_than" for e in errors)


def test_pagination_limit_offset_and_total() -> None:
    """Test that pagination with limit and offset parameters works correctly."""
    with _client() as client:
        payloads: list[dict[str, str | None]] = [
            {"name": "A", "description": None},
            {"name": "B", "description": "b"},
            {"name": "C", "description": ""},
        ]
        ids: list[int] = []
        for p in payloads:
            r = client.post("/v1/items", json=p)
            assert r.status_code == 201
            ids.append(r.json()["id"])

        r = client.get("/v1/items", params={"limit": 2, "offset": 1})
        assert r.status_code == 200
        data = r.json()
        assert data["total"] == 5
        assert len(data["items"]) == 2

        returned_ids = [it["id"] for it in data["items"]]
        assert returned_ids == [2, 3]


def test_update_item_same_name_different_description() -> None:
    """Test that an item can be updated with a new description but the same name."""
    with _client() as client:
        update_payload = {"name": "Item1", "description": "An updated description"}

        r = client.put("/v1/items/1", json=update_payload)

        assert r.status_code == 200
        updated_item = r.json()
        assert updated_item["id"] == 1
        assert updated_item["name"] == "Item1"
        assert updated_item["description"] == "An updated description"

        r_get = client.get("/v1/items/1")
        assert r_get.status_code == 200
        assert r_get.json()["description"] == "An updated description"
