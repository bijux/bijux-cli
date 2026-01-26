# SPDX-License-Identifier: Apache-2.0
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

from collections.abc import AsyncIterator, Awaitable, Callable
from contextlib import asynccontextmanager
import json
import logging
import threading
from typing import Annotated, Any, Protocol, runtime_checkable

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
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.params import Depends as DependsMarker
from fastapi.responses import JSONResponse
from pydantic import AnyUrl, BaseModel, ConfigDict, Field, field_validator

logger = logging.getLogger("bijux_cli.api.http")


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

    model_config = ConfigDict(
        extra="forbid",
    )

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
    def validate_and_normalize_name(cls: type[ItemIn], v: str) -> str:  # noqa: N805
        """Strips whitespace and ensures the name is not empty.

        Args:
            v: The input string for the item's name.

        Returns:
            The validated and stripped name.

        Raises:
            ValueError: If the name is empty or contains only whitespace.
        """
        for ch in v:
            codepoint = ord(ch)
            if codepoint < 0x20 or 0x7F <= codepoint <= 0x9F:
                raise ValueError("name must not contain control characters")
            if 0xD800 <= codepoint <= 0xDFFF:
                raise ValueError("name must not contain surrogate code points")
        stripped_v = v.strip()
        if not stripped_v:
            raise ValueError("name must not be empty or contain only whitespace")
        return stripped_v


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

    def find_by_name(self, name: str) -> Item | None:
        """Returns an item by its name if it exists, otherwise None."""
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
            key = data.name.strip().lower()
            if key in self._name_index:
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
            item = Item(id=item_id, name=data.name, description=data.description)
            self._items[item_id] = item
            self._name_index[key] = item_id
            logger.info("Created item: %s", item)
            return item

    def update(self, item_id: int, data: ItemIn) -> Item:
        """Update an existing item.

        Args:
            item_id (int): The unique identifier of the item to update.
            data (ItemIn): The new values for the item.

        Returns:
            The updated item.

        Raises:
            HTTPException: If the item does not exist (HTTP 404) or if the new name
                conflicts with another item (HTTP 409).
        """
        with self._lock:
            existing = self._items.get(item_id)
            if existing is None:
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

            old_key = existing.name.strip().lower()
            new_key = data.name.strip().lower()

            if new_key != old_key and new_key in self._name_index:
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

            updated = Item(id=item_id, name=data.name, description=data.description)
            self._items[item_id] = updated
            if new_key != old_key:
                self._name_index.pop(old_key, None)
                self._name_index[new_key] = item_id
            logger.info("Updated item id=%s", item_id)
            return updated

    def delete(self, item_id: int) -> None:
        """Delete an item by its unique ID.

        Args:
            item_id: The unique ID of the item to delete.
        """
        with self._lock:
            existing = self._items.pop(item_id, None)
            if existing is None:
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
            self._name_index.pop(existing.name.strip().lower(), None)
            logger.info("Deleted item id=%s", item_id)

    def reset(self) -> None:
        """Resets the store to its initial empty state."""
        with self._lock:
            self._items.clear()
            self._name_index.clear()
            self._next_id = 1
            logger.info("Store reset")

    def prepopulate(self, data: list[dict[str, Any]]) -> None:
        """Prepopulates the store with a list of items.

        Args:
            data: A list of dictionaries, where each dictionary contains
                the data for a new item.
        """
        with self._lock:
            for entry in data:
                self.create(ItemIn(**entry))

    def find_by_name(self, name: str) -> Item | None:
        """Lookup an item by its name (case-insensitive, trimmed)."""
        with self._lock:
            key = name.strip().lower()
            item_id = self._name_index.get(key)
            return self._items.get(item_id) if item_id is not None else None


def get_store() -> ItemStoreProtocol:
    """A FastAPI dependency to provide the `ItemStoreProtocol` instance."""
    return store


def get_item_or_404(
    item_id: int = Path(..., ge=1),
    store: ItemStoreProtocol = Depends(get_store),  # noqa: B008
) -> Item:
    """A dependency that retrieves an item by ID or raises a 404."""
    return store.get(item_id)


def reject_duplicate_query_params(*params: str) -> DependsMarker:
    """Create a dependency that rejects duplicate query parameters (HTTP 422).

    Args:
      *params: Names of query parameters that must not appear more than once.

    Returns:
      fastapi.params.Depends: A dependency marker that, when executed at
      request time, raises ``HTTPException`` (422) if any listed parameter
      appears more than once.

    Raises:
      HTTPException: Emitted at request time if duplicates are detected.
    """

    async def _dep(request: Request) -> None:
        """Raise an HTTPException if specific query parameters are duplicated.

        This function is designed to be used as a FastAPI dependency. It checks an
        iterable of parameter names (assumed to be in the parent scope's `params`
        variable) to ensure they are not repeated in the request's query string.

        Args:
            request: The incoming FastAPI/Starlette request object.

        Raises:
            HTTPException: An exception with a 422 status code and a
                problem+json body if any of the specified query parameters
                are found more than once.
        """
        duplicates = [p for p in params if len(request.query_params.getlist(p)) > 1]
        if duplicates:
            detail = f"Duplicate query params found: {', '.join(sorted(duplicates))}"
            raise HTTPException(
                status_code=422,
                detail=Problem(
                    type=AnyUrl("https://bijux-cli.dev/docs/errors/validation-error"),
                    title="Validation error",
                    status=422,
                    detail=detail,
                    instance=str(request.url),
                ).model_dump(mode="json"),
            )

    return DependsMarker(_dep)


router = APIRouter(prefix="/v1")


def require_accept_json(request: Request) -> None:
    """Reject requests that don't accept application/json (HTTP 406).

    Schemathesis' negative-data checks may send unsupported Accept headers.
    If the client doesn't accept JSON (and not */*), respond with 406.
    """
    accept = request.headers.get("accept", "*/*").lower()
    if "*/*" in accept or "application/json" in accept:
        return
    raise HTTPException(
        status_code=status.HTTP_406_NOT_ACCEPTABLE,
        detail=Problem(
            type=AnyUrl("https://bijux-cli.dev/docs/errors/not-acceptable"),
            title="Not Acceptable",
            status=status.HTTP_406_NOT_ACCEPTABLE,
            detail="Set 'Accept: application/json' for this endpoint",
            instance=str(request.url),
        ).model_dump(mode="json"),
    )


def allow_only(*allowed: str) -> Callable[[Request], Awaitable[None]]:
    """Create a dependency that rejects unknown query parameters (HTTP 422).

    Args:
      *allowed (str): The set of query parameter names that are permitted.

    Returns:
      Callable[[Request], Awaitable[None]]: An async dependency suitable for
      FastAPI's ``dependencies=[...]``. It raises an ``HTTPException`` with
      status 422 if the request includes parameters outside the allowlist.

    Raises:
      HTTPException: Emitted by the returned dependency at request time when
        unknown query parameters are present (422 Unprocessable Entity).
    """
    allowed_set: set[str] = set(allowed)

    async def _dep(request: Request) -> None:
        """Raise an HTTPException if unknown query parameters are present.

        This function is designed to be used as a FastAPI dependency. It validates
        that the request's query string contains only parameters from a pre-defined
        set of allowed names, assumed to be in the parent scope's `allowed_set`
        variable.

        Args:
            request: The incoming FastAPI/Starlette request object.

        Raises:
            HTTPException: An exception with a 422 status code and a
                problem+json body if any query parameters are found that are
                not in the `allowed_set`.
        """
        extras = set(request.query_params.keys()) - allowed_set
        if extras:
            raise HTTPException(
                status_code=422,
                detail=Problem(
                    type=AnyUrl("https://bijux-cli.dev/docs/errors/validation-error"),
                    title="Validation error",
                    status=422,
                    detail=f"Unknown query params: {', '.join(sorted(extras))}",
                    instance=str(request.url),
                ).model_dump(mode="json"),
            )

    return _dep


@router.get(
    "/items",
    response_model=ItemListResponse,
    summary="List items",
    description="List all items with pagination.",
    tags=["Items"],
    responses={406: {"model": Problem}, 422: {"model": Problem}},
    dependencies=[
        Depends(require_accept_json),
        Depends(allow_only("limit", "offset")),
        reject_duplicate_query_params("limit", "offset"),
    ],
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
    responses={404: {"model": Problem}, 406: {"model": Problem}},
    tags=["Items"],
    dependencies=[Depends(require_accept_json)],
)
def get_item(
    item: Item = Depends(get_item_or_404),  # noqa: B008
) -> Item:
    """Retrieves a single item by its ID.

    This endpoint uses a dependency (`get_item_or_404`) to fetch the item,
    ensuring that a 404 response is returned if the item does not exist.

    Args:
        item (Item): The item retrieved by the `get_item_or_404` dependency.

    Returns:
        Item: The requested item.
    """
    return item


@router.post(
    "/items",
    response_model=Item,
    status_code=status.HTTP_201_CREATED,
    summary="Create item",
    description="Create a new item.",
    responses={
        200: {
            "model": Item,
            "description": "Item already exists; existing resource returned",
        },
        406: {"model": Problem},
        409: {"model": Problem},
        422: {"model": Problem},
    },
    tags=["Items"],
    dependencies=[Depends(require_accept_json)],
)
def create_item(
    response: Response,
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
    existing = store.find_by_name(item.name)
    if existing is not None:
        response.status_code = status.HTTP_200_OK
        return existing
    return store.create(item)


@router.put(
    "/items/{item_id}",
    response_model=Item,
    summary="Update item",
    description="Update an existing item.",
    responses={
        406: {"model": Problem},
        404: {"model": Problem},
        409: {"model": Problem},
        422: {"model": Problem},
    },
    tags=["Items"],
    dependencies=[Depends(require_accept_json)],
)
def update_item(
    item: Annotated[Item, Depends(get_item_or_404)],
    update_data: Annotated[ItemIn, Body(...)],
    store: Annotated[ItemStoreProtocol, Depends(get_store)],
) -> Item:
    """Update an existing item.

    Args:
      item (Item): The current item resolved from the path parameter,
        injected by ``get_item_or_404``.
      update_data (ItemIn): The new values for the item (request body).
      store (ItemStoreProtocol): The item store implementation (injected).

    Returns:
      The updated item.

    Raises:
      HTTPException: If the item does not exist (404) or if the new name
        conflicts with another item (409). These are raised by the dependency
        or the store layer.
      RequestValidationError: If the path/body validation fails (422). Handled
        by the global validation exception handler.
    """
    return store.update(item.id, update_data)


@router.delete(
    "/items/{item_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="Delete item",
    description="Delete an item by its ID.",
    responses={
        406: {"model": Problem},
        404: {"model": Problem},
        422: {"model": Problem},
    },
    tags=["Items"],
    dependencies=[Depends(require_accept_json)],
)
def delete_item(
    item: Annotated[Item, Depends(get_item_or_404)],
    store: Annotated[ItemStoreProtocol, Depends(get_store)],
) -> Response:
    """Delete an item by its unique ID.

    The target item is resolved by the `get_item_or_404` dependency before this
    handler runs.

    Args:
        item: The item to delete, injected by `get_item_or_404`.
        store: The item store implementation, injected.

    Returns:
        Response: Empty body with **204 No Content** on successful deletion.

    Raises:
        HTTPException: 404 if the item does not exist (raised by the dependency).
    """
    store.delete(item.id)
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
    errors = jsonable_encoder(exc.errors())
    logger.warning("Validation error: %s", errors)
    return JSONResponse(
        status_code=422,
        content={
            "type": "https://bijux-cli.dev/docs/errors/validation-error",
            "title": "Validation error",
            "status": 422,
            "detail": json.dumps(errors),
            "instance": str(request.url),
        },
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
    logger.warning("HTTP error: %s %s", exc.status_code, exc.detail)
    return JSONResponse(status_code=exc.status_code, content=exc.detail)
