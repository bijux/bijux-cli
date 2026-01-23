# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Pytest configuration and test_fixtures for the Bijux CLI HTTP API tests."""

from __future__ import annotations

from collections.abc import Generator
import uuid

import httpx
import pytest
from starlette.testclient import TestClient

from bijux_cli.api.http import app

BASE_URL = "http://testserver/v1"


@pytest.fixture(scope="module")
def client() -> Generator[TestClient, None, None]:
    """Provide an in-process TestClient with the app started (incl. lifespan)."""
    with TestClient(app, base_url=BASE_URL) as c:
        yield c


@pytest.fixture
def create_test_item(client: httpx.Client) -> Generator[int, None, None]:
    """Create a test item and clean it up after the test.

    This fixture POSTs a new item to the API at the beginning of a test
    and DELETEEs it upon completion, ensuring a clean state.

    Args:
        client: The `httpx.Client` fixture.

    Yields:
        The ID of the newly created test item.
    """
    name = f"Test Item {uuid.uuid4()}"
    payload = {"name": name, "description": "A test description"}
    response = client.post("/items", json=payload)
    assert response.status_code == 201, f"Failed to create test item: {response.text}"
    item_id = response.json()["id"]
    try:
        yield item_id
    finally:
        client.delete(f"/items/{item_id}")
