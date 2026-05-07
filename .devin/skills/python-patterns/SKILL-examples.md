# Python Patterns - Code Examples

Code examples for type hints, Pydantic integration, FastAPI patterns, and async testing.

## Common Type Patterns

```python
from typing import Optional, Callable

# Optional → might be None
def find_user(id: int) -> Optional[User]:
    ...

# Union → one of multiple types
def process(data: str | dict) -> None:
    ...

# Generic collections
def get_items() -> list[Item]:
    ...

def get_mapping() -> dict[str, int]:
    ...

# Callable
def apply(fn: Callable[[int], str]) -> str:
    ...
```

## FastAPI + Pydantic Integration

```python
# Request validation
@app.post("/users")
async def create(user: UserCreate) -> UserResponse:
    # user is already validated
    ...

# Response serialization
# Return type becomes response schema
```

## Async Testing with pytest

```python
import pytest
from httpx import AsyncClient

@pytest.mark.asyncio
async def test_endpoint():
    async with AsyncClient(app=app, base_url="http://test") as client:
        response = await client.get("/users")
        assert response.status_code == 200
```

## Related Resources

See `@[skills/python-patterns/SKILL.md]` for framework selection, async vs sync decisions, type hints strategy, and project structure principles.

See `@[skills/python-patterns/SKILL-advanced.md]` for Django principles, FastAPI patterns, background tasks, error handling, testing strategies, and decision checklists.