# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a self-contained FastAPI application for a CRUD API.

This module defines a complete HTTP API for managing "Item" resources using
the FastAPI framework. It includes all necessary components for a functional
web service.

Services:
    * **Pydantic Models:** `ItemIn`, `Item`, and response models for data
        validation and serialization.
    * **Storage Layer:** A formal `ItemStoreProtocol` and a concrete,
        thread-safe `InMemoryItemStore` implementation.
    * **API Endpoints:** A FastAPI `APIRouter` with path operations for all
        CRUD (Create, Read, Update, Delete) actions.
    * **Application Lifecycle:** A `lifespan` manager to prepopulate and
        clear the data store on startup and shutdown.
"""

from __future__ import annotations

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
import logging
import threading
from typing import Any, Protocol, runtime_checkable

from fastapi import (
    APIRouter,
    Body,
    Depends,
    FastAPI,
    HTTPException,
    Path,
    Query,
    Request,
    Response,
    status,
)
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from pydantic import AnyUrl, BaseModel, Field, field_validator

logger = logging.getLogger("bijux_cli.httpapi")
logging.basicConfig(level=logging.INFO)


class Problem(BaseModel):
    """Defines a standard RFC 7807 problem details response.

    Attributes:
        type (AnyUrl): A URI reference that identifies the problem type.
        title (str): A short, human-readable summary of the problem type.
        status (int): The HTTP status code.
        detail (str): A human-readable explanation specific to this occurrence.
        instance (str): A URI reference that identifies the specific occurrence.
    """

    type: AnyUrl = Field(
        default=AnyUrl("about:blank"),
        description="A URI reference that identifies the problem type.",
    )
    title: str = Field(..., description="A short, human-readable summary.")
    status: int = Field(..., description="The HTTP status code.")
    detail: str = Field(..., description="A human-readable explanation.")
    instance: str = Field(..., description="A URI reference for this occurrence.")


class ItemIn(BaseModel):
    """Defines the input model for creating or updating an item.

    Attributes:
        name (str): The name of the item.
        description (str | None): An optional description for the item.
    """

    name: str = Field(
        ...,
        min_length=1,
        max_length=100,
        json_schema_extra={"example": "Sample"},
    )
    description: str | None = Field(
        None,
        max_length=500,
        json_schema_extra={"example": "Details about this item"},
    )

    @field_validator("name")
    @classmethod
    def normalize_name(cls: type[ItemIn], v: str) -> str:  # noqa: N805
        """Strips leading/trailing whitespace from the name field."""
        return v.strip()


class Item(ItemIn):
    """Defines the full item model, including its unique identifier.

    Attributes:
        id (int): The unique identifier for the item.
        name (str): The name of the item.
        description (str | None): An optional description for the item.
    """

    id: int = Field(..., json_schema_extra={"example": 1})


class ItemListResponse(BaseModel):
    """Defines the response model for a paginated list of items.

    Attributes:
        items (list[Item]): The list of items on the current page.
        total (int): The total number of items available.
    """

    items: list[Item]
    total: int


@runtime_checkable
class ItemStoreProtocol(Protocol):
    """Defines the contract for an item storage service."""

    def list_items(self, limit: int, offset: int) -> tuple[list[Item], int]:
        """Lists items with pagination."""
        ...

    def get(self, item_id: int) -> Item:
        """Gets an item by its unique ID."""
        ...

    def create(self, data: ItemIn) -> Item:
        """Creates a new item."""
        ...

    def update(self, item_id: int, data: ItemIn) -> Item:
        """Updates an existing item."""
        ...

    def delete(self, item_id: int) -> None:
        """Deletes an item by its unique ID."""
        ...

    def reset(self) -> None:
        """Resets the store to its initial empty state."""
        ...

    def prepopulate(self, data: list[dict[str, Any]]) -> None:
        """Prepopulates the store with a list of items."""
        ...


class InMemoryItemStore(ItemStoreProtocol):
    """A thread-safe, in-memory implementation of the `ItemStoreProtocol`.

    Attributes:
        _items (dict): The main dictionary storing items by their ID.
        _name_index (dict): An index to enforce unique item names.
        _lock (threading.RLock): A lock to ensure thread-safe operations.
        _next_id (int): A counter for generating new item IDs.
    """

    def __init__(self) -> None:
        """Initializes the in-memory item store."""
        self._items: dict[int, Item] = {}
        self._name_index: dict[str, int] = {}
        self._lock = threading.RLock()
        self._next_id = 1

    def list_items(self, limit: int, offset: int) -> tuple[list[Item], int]:
        """Lists items with pagination in a thread-safe manner.

        Args:
            limit (int): The maximum number of items to return.
            offset (int): The starting index for the items to return.

        Returns:
            A tuple containing the list of items and the total number of items.
        """
        with self._lock:
            items = list(self._items.values())
            return items[offset : offset + limit], len(items)

    def get(self, item_id: int) -> Item:
        """Gets an item by its unique ID.

        Args:
            item_id (int): The ID of the item to retrieve.

        Returns:
            The requested item.

        Raises:
            HTTPException: With status 404 if the item is not found.
        """
        with self._lock:
            item = self._items.get(item_id)
            if not item:
                raise HTTPException(
                    status_code=status.HTTP_404_NOT_FOUND,
                    detail=Problem(
                        type=AnyUrl("https://bijux-cli.dev/docs/errors/not-found"),
                        title="Not found",
                        status=status.HTTP_404_NOT_FOUND,
                        detail="Item not found",
                        instance=f"/v1/items/{item_id}",
                    ).model_dump(mode="json"),
                )
            return item

    def create(self, data: ItemIn) -> Item:
        """Creates a new item.

        Args:
            data (ItemIn): The data for the new item.

        Returns:
            The newly created item, including its generated ID.

        Raises:
            HTTPException: With status 409 if an item with the same name exists.
        """
        with self._lock:
            name = data.name  # Already stripped by validator
            if name in self._name_index:
                raise HTTPException(
                    status_code=status.HTTP_409_CONFLICT,
                    detail=Problem(
                        type=AnyUrl("https://bijux-cli.dev/docs/errors/conflict"),
                        title="Conflict",
                        status=status.HTTP_409_CONFLICT,
                        detail="Item with this name already exists",
                        instance="/v1/items",
                    ).model_dump(mode="json"),
                )
            item_id = self._next_id
            self._next_id += 1
            item = Item(id=item_id, name=name, description=data.description)
            self._items[item_id] = item
            self._name_index[name] = item_id
            logger.info(f"Created item: {item}")
            return item

    def update(self, item_id: int, data: ItemIn) -> Item:
        """Updates an existing item.

        Args:
            item_id (int): The ID of the item to update.
            data (ItemIn): The new data for the item.

        Returns:
            The updated item.

        Raises:
            HTTPException: With status 404 if the item is not found, or 409
                if the new name conflicts with another existing item.
        """
        with self._lock:
            item = self._items.get(item_id)
            if not item:
                raise HTTPException(
                    status_code=status.HTTP_404_NOT_FOUND,
                    detail=Problem(
                        type=AnyUrl("https://bijux-cli.dev/docs/errors/not-found"),
                        title="Not found",
                        status=status.HTTP_404_NOT_FOUND,
                        detail="Item not found",
                        instance=f"/v1/items/{item_id}",
                    ).model_dump(mode="json"),
                )
            name = data.name  # Already stripped by validator
            if name != item.name and name in self._name_index:
                raise HTTPException(
                    status_code=status.HTTP_409_CONFLICT,
                    detail=Problem(
                        type=AnyUrl("https://bijux-cli.dev/docs/errors/conflict"),
                        title="Conflict",
                        status=status.HTTP_409_CONFLICT,
                        detail="Item with this name already exists",
                        instance=f"/v1/items/{item_id}",
                    ).model_dump(mode="json"),
                )
            if name != item.name:
                del self._name_index[item.name]
                self._name_index[name] = item_id
            updated = Item(id=item_id, name=name, description=data.description)
            self._items[item_id] = updated
            logger.info(f"Updated item: {updated}")
            return updated

    def delete(self, item_id: int) -> None:
        """Deletes an item by its unique ID.

        Args:
            item_id (int): The ID of the item to delete.

        Raises:
            HTTPException: With status 404 if the item is not found.
        """
        with self._lock:
            item = self._items.pop(item_id, None)
            if not item:
                raise HTTPException(
                    status_code=status.HTTP_404_NOT_FOUND,
                    detail=Problem(
                        type=AnyUrl("https://bijux-cli.dev/docs/errors/not-found"),
                        title="Not found",
                        status=status.HTTP_404_NOT_FOUND,
                        detail="Item not found",
                        instance=f"/v1/items/{item_id}",
                    ).model_dump(mode="json"),
                )
            del self._name_index[item.name]
            logger.info(f"Deleted item id={item_id}")

    def reset(self) -> None:
        """Resets the store to its initial empty state."""
        with self._lock:
            self._items.clear()
            self._name_index.clear()
            self._next_id = 1
            logger.info("Store reset")

    def prepopulate(self, data: list[dict[str, Any]]) -> None:
        """Prepopulates the store with a list of items."""
        with self._lock:
            for entry in data:
                self.create(ItemIn(**entry))


def get_store() -> ItemStoreProtocol:
    """A FastAPI dependency to provide the `ItemStoreProtocol` instance."""
    return store


router = APIRouter(prefix="/v1")


@router.get(
    "/items",
    response_model=ItemListResponse,
    summary="List items",
    description="List all items with pagination.",
    tags=["Items"],
)
def list_items(
    limit: int = Query(10, ge=1, le=100),
    offset: int = Query(0, ge=0),
    store: ItemStoreProtocol = Depends(get_store),  # noqa: B008
) -> ItemListResponse:
    """Retrieves a paginated list of items.

    Args:
        limit (int): The maximum number of items per page.
        offset (int): The starting offset for the item list.
        store (ItemStoreProtocol): The dependency-injected item store.

    Returns:
        ItemListResponse: An object containing the list of items and total count.
    """
    items, total = store.list_items(limit, offset)
    return ItemListResponse(items=items, total=total)


@router.get(
    "/items/{item_id}",
    response_model=Item,
    summary="Get item",
    description="Get a single item by its ID.",
    responses={404: {"model": Problem}},
    tags=["Items"],
)
def get_item(
    item_id: int = Path(..., gt=0),
    store: ItemStoreProtocol = Depends(get_store),  # noqa: B008
) -> Item:
    """Retrieves a single item by its ID.

    Args:
        item_id (int): The unique identifier of the item to retrieve.
        store (ItemStoreProtocol): The dependency-injected item store.

    Returns:
        Item: The requested item.
    """
    return store.get(item_id)


@router.post(
    "/items",
    response_model=Item,
    status_code=status.HTTP_201_CREATED,
    summary="Create item",
    description="Create a new item.",
    responses={409: {"model": Problem}},
    tags=["Items"],
)
def create_item(
    item: ItemIn = Body(...),  # noqa: B008
    store: ItemStoreProtocol = Depends(get_store),  # noqa: B008
) -> Item:
    """Creates a new item.

    Args:
        item (ItemIn): The data for the new item from the request body.
        store (ItemStoreProtocol): The dependency-injected item store.

    Returns:
        Item: The newly created item, including its server-generated ID.
    """
    return store.create(item)


@router.put(
    "/items/{item_id}",
    response_model=Item,
    summary="Update item",
    description="Update an existing item.",
    responses={404: {"model": Problem}, 409: {"model": Problem}},
    tags=["Items"],
)
def update_item(
    item_id: int = Path(..., gt=0),
    item: ItemIn = Body(...),  # noqa: B008
    store: ItemStoreProtocol = Depends(get_store),  # noqa: B008
) -> Item:
    """Updates an existing item by its ID.

    Args:
        item_id (int): The unique identifier of the item to update.
        item (ItemIn): The new data for the item from the request body.
        store (ItemStoreProtocol): The dependency-injected item store.

    Returns:
        Item: The updated item.
    """
    return store.update(item_id, item)


@router.delete(
    "/items/{item_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="Delete item",
    description="Delete an item by its ID.",
    responses={404: {"model": Problem}},
    tags=["Items"],
)
def delete_item(
    item_id: int = Path(..., gt=0),
    store: ItemStoreProtocol = Depends(get_store),  # noqa: B008
) -> Response:
    """Deletes an item by its ID.

    Args:
        item_id (int): The unique identifier of the item to delete.
        store (ItemStoreProtocol): The dependency-injected item store.

    Returns:
        Response: An empty response with a 204 No Content status code.
    """
    store.delete(item_id)
    return Response(status_code=status.HTTP_204_NO_CONTENT)


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    """Manages the application's lifespan events for startup and shutdown.

    On startup, this context manager resets and prepopulates the in-memory
    store with demo data. On shutdown, it resets the store again.

    Args:
        app (FastAPI): The FastAPI application instance.

    Yields:
        None: Yields control to the application while it is running.
    """
    store.reset()
    store.prepopulate(
        [
            {"name": "Item One", "description": "Description one"},
            {"name": "Item Two", "description": "Description two"},
        ]
    )
    logger.info("Prepopulated store with demo items")
    yield
    store.reset()
    logger.info("Store reset on shutdown")


store = InMemoryItemStore()
app = FastAPI(
    title="Bijux CLI API",
    version="1.0.0",
    description="High-quality demo API for educational/reference purposes.",
    lifespan=lifespan,
)
app.include_router(router)


@app.get("/health", summary="Health check", tags=["Health"])
async def health() -> dict[str, str]:
    """Lightweight readiness probe used by Makefile `api-test`."""
    return {"status": "ok"}


@app.exception_handler(RequestValidationError)
async def validation_exception_handler(
    request: Request, exc: RequestValidationError
) -> JSONResponse:
    """A custom exception handler for `RequestValidationError`.

    This handler intercepts validation errors from FastAPI and formats them
    into a standard `JSONResponse` with a 422 status code.

    Args:
        request (Request): The incoming request.
        exc (RequestValidationError): The validation exception.

    Returns:
        JSONResponse: A JSON response detailing the validation error.
    """
    logger.warning(f"Validation error: {exc.errors()}")
    return JSONResponse(
        status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
        content=Problem(
            type=AnyUrl("https://bijux-cli.dev/docs/errors/validation-error"),
            title="Validation error",
            status=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail=str(exc),
            instance=str(request.url),
        ).model_dump(mode="json"),
    )


@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException) -> JSONResponse:
    """A custom exception handler for `HTTPException`.

    This handler intercepts FastAPI's standard HTTP exceptions and ensures they
    are logged and returned in the standard JSON error format.

    Args:
        request (Request): The incoming request.
        exc (HTTPException): The HTTP exception.

    Returns:
        JSONResponse: A JSON response detailing the HTTP error.
    """
    logger.warning(f"HTTP error: {exc.status_code} {exc.detail}")
    return JSONResponse(status_code=exc.status_code, content=exc.detail)
