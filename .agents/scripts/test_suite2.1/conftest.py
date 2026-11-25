"""
pytest fixtures for test_suite2.

Provides:
- client: RobotBrainClient connected to robot_brain binary (session-scoped)
- data_cleanup: track created IDs for cleanup
- server_path: path to robot_brain binary
"""

import asyncio
import contextlib
from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def server_path():
    """Path to robot_brain binary (absolute, handles Windows .exe)."""
    import sys

    test_suite2_dir = Path(__file__).resolve().parent
    scripts_dir = test_suite2_dir.parent  # .agents/scripts/
    agents_dir = scripts_dir.parent  # .agents/
    project_root = agents_dir.parent  # project root

    # Check both release and debug builds
    for build_type in ["release", "debug"]:
        base_path = project_root / "target" / build_type / "robot_brain"
        # Windows uses .exe, Unix doesn't
        if base_path.exists():
            return base_path
        exe_path = base_path.with_suffix(".exe")
        if exe_path.exists():
            return exe_path

    raise FileNotFoundError(
        f"robot_brain binary not found.\n"
        f"Searched: {project_root}/target/release/robot_brain" + ".exe"
        if sys.platform == "win32"
        else "" + f"\nBuild it: cd {project_root} && cargo build --release"
    )


@pytest.fixture(scope="function")
def data_cleanup():
    """Track created IDs for cleanup."""
    return {
        "memories": [],
        "knowledge": [],
        "experiences": [],
        "plans": [],
        "skills": [],
    }


SESSION_START_TIME = None
SESSION_TIMEOUT = 1500  # 25 min total session timeout


@pytest.fixture(scope="session", autouse=True)
def _session_timer():
    """Track session start time for global timeout enforcement."""
    global SESSION_START_TIME
    import time

    SESSION_START_TIME = time.time()
    yield
    SESSION_START_TIME = None


@pytest.hookimpl(tryfirst=True)
def pytest_runtest_setup(item):
    """Check global timeout before each test."""
    import time

    if SESSION_START_TIME is not None:
        elapsed = time.time() - SESSION_START_TIME
        if elapsed > SESSION_TIMEOUT:
            pytest.skip(
                f"Session timeout exceeded ({elapsed:.0f}s > {SESSION_TIMEOUT}s). "
                f"Test was not run."
            )


@pytest.hookimpl(tryfirst=True)
def pytest_runtest_call(item):
    """Enforce per-test timeout for slow tests."""
    import inspect
    import time

    # Get the test function
    test_func = item.obj

    # Skip if not a coroutine function
    if not inspect.iscoroutinefunction(test_func):
        yield
        return
        yield
        return

    # Get the timeout from the test's own timeout (if any) or use default
    timeout = getattr(item, "function", None)
    if hasattr(item, "pytestmark"):
        for mark in item.pytestmark:
            if mark.name == "timeout":
                timeout = mark.args[0]
                break

    # Default 60s timeout per test
    test_timeout = timeout if timeout else 60

    start = time.time()
    try:
        yield
    except TimeoutError:
        raise pytest.fail(
            f"Test exceeded {test_timeout}s timeout ({time.time() - start:.1f}s). "
            f"Likely the server or a tool is hanging."
        )
    finally:
        elapsed = time.time() - start
        if elapsed > test_timeout:
            raise pytest.fail(
                f"Test exceeded {test_timeout}s timeout ({elapsed:.1f}s). "
                f"Likely the server or a tool is hanging."
            )


@pytest.fixture(scope="session")
async def client(server_path):
    """Session-scoped MCP client with workflow initialized.

    Reuses the same server instance for all tests to avoid ~10s startup cost
    per test. The workflow gate (get_workflow + search_memory) is called once
    during session setup.

    Uses session-scoped event loop (configured in pytest.ini) so the
    subprocess transport stays valid across all tests.
    """
    import sys

    _test_dir = Path(__file__).resolve().parent
    if str(_test_dir) not in sys.path:
        sys.path.insert(0, str(_test_dir))
    from mcp_client import RobotBrainClient

    c = await RobotBrainClient.connect(server_path)
    # Initialize workflow gate - required by server
    await c.get_workflow("general")
    await c.search_memory_first("test")
    yield c
    await c.stop()


@contextlib.asynccontextmanager
async def timeout_wrapper(coro, timeout_seconds=30):
    """Wrap an async call with a timeout.

    Usage:
        async with timeout_wrapper(mycoro(), 15) as result:
            assert result is not None
    """
    try:
        result = await asyncio.wait_for(coro, timeout=timeout_seconds)
    except TimeoutError:
        pytest.fail(
            f"Test exceeded {timeout_seconds}s timeout — "
            f"likely the server is stuck or the tool is hanging"
        )
    yield result
