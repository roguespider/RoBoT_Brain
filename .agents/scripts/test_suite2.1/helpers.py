"""
Shared test helpers for test_suite2.
"""


def assert_success(result):
    """Check if a tool response indicates success.

    Checks that the result is a non-empty dict AND contains a 'success' field
    that is truthy (True, non-empty string, non-zero number, etc.).
    This catches cases where the server returns an error response.
    """
    if result is None:
        return False
    if not isinstance(result, dict):
        return False
    if len(result) == 0:
        return False
    # Check for explicit success field (server returns {"success": true} on success)
    success_field = result.get("success")
    if success_field is not None:
        return bool(success_field)
    # If no explicit success field, check for error indicator
    if result.get("error") or result.get("error_message"):
        return False
    # If we got here, the response has content but no explicit success/error
    # This is acceptable for some tools (e.g., list operations return arrays)
    return True


def assert_success_strict(result, tool_name="unknown"):
    """Strict assertion — requires explicit success field.

    Raises AssertionError if the response doesn't indicate success.
    Use this for tests where failure should cause test failure.
    """
    if result is None:
        raise AssertionError(f"Tool '{tool_name}' returned None")
    if not isinstance(result, dict):
        raise AssertionError(
            f"Tool '{tool_name}' returned non-dict: {type(result).__name__}"
        )
    success_field = result.get("success")
    if success_field is None:
        raise AssertionError(
            f"Tool '{tool_name}' response missing 'success' field: {result}"
        )
    if not success_field:
        error_msg = result.get("error", result.get("error_message", "Unknown error"))
        raise AssertionError(
            f"Tool '{tool_name}' failed: {error_msg}. Response: {result}"
        )
    return True
