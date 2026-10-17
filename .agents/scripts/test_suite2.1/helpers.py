"""
Shared test helpers for test_suite2.
"""


def assert_success(result):
    """Check if a tool response indicates success."""
    return (
        result.get("success") is True
        or result.get("found") is True
        or result.get("id") is not None
        or result.get("created") is True
        or result.get("created_or_updated") is True
        or result.get("added") is True
    )
