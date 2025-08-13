# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Pytest configuration and test_fixtures for the Bijux CLI HTTP API tests."""

from __future__ import annotations

from collections.abc import Generator

import httpx
import pytest

FASTAPI_HOST = "0.0.0.0"  # noqa: S104
FASTAPI_PORT = 8000
BASE_URL = f"http://{FASTAPI_HOST}:{FASTAPI_PORT}/v1"


@pytest.fixture(scope="module")
def client() -> httpx.Client:
    """Provide an `httpx` client for making API requests.

    Returns:
        An `httpx.Client` instance configured with the base URL of the API.
    """
    return httpx.Client(base_url=BASE_URL, timeout=5.0)


@pytest.fixture
def create_test_item(client: httpx.Client) -> Generator[int, None, None]:
    """Create a test item and clean it up after the test.

    This fixture POSTs a new item to the API at the beginning of a test
    and DELETEs it upon completion, ensuring a clean state.

    Args:
        client: The `httpx.Client` fixture.

    Yields:
        The ID of the newly created test item.
    """
    payload = {"name": "Test Item", "description": "A test description"}
    response = client.post("/items", json=payload)
    assert response.status_code == 201, f"Failed to create test item: {response.text}"
    item = response.json()
    item_id = item["id"]
    yield item_id
    client.delete(f"/items/{item_id}")
