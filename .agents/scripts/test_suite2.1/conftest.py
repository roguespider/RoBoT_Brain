"""
pytest fixtures for test_suite2.

Provides:
- client: RobotBrainClient connected to robot_brain binary
- data_cleanup: track created IDs for cleanup
- server_path: path to robot_brain binary
"""

import asyncio
from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def event_loop():
    """Create event loop for the test session."""
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


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


@pytest.fixture(scope="function")
async def client(server_path, data_cleanup):
    """Connected MCP client with workflow initialized."""
    import sys
    from pathlib import Path

    _test_dir = Path(__file__).resolve().parent
    if str(_test_dir) not in sys.path:
        sys.path.insert(0, str(_test_dir))
    from mcp_client import RobotBrainClient

    c = await RobotBrainClient.connect(server_path)

    # Initialize workflow gate - these are required by the server
    await c.get_workflow("general")
    await c.search_memory_first("test")

    yield c
    await c.stop()
