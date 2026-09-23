"""
MCP client for test_suite2.

Connects to robot_brain via stdio using LINE-BASED JSON-RPC protocol.
Same protocol as Rust TestMcpClient — sends JSON + newline, reads line-by-line.
NO source code access — only the compiled binary is tested.
"""

import asyncio
import asyncio.subprocess
import json
from pathlib import Path
from typing import cast


class RobotBrainClient:
    """MCP client using robot_brain's line-based JSON-RPC protocol."""

    def __init__(self, server_path: Path):
        self.server_path = server_path
        self._process: asyncio.subprocess.Process | None = None
        self._request_id = 0
        self._workflow_retrieved = False
        self._memory_searched = False

    @classmethod
    async def connect(cls, server_path: Path | None = None) -> "RobotBrainClient":
        """Create and connect to robot_brain."""
        project_root = None
        if server_path is None:
            test_suite2_dir = Path(__file__).resolve().parent
            scripts_dir = test_suite2_dir.parent
            agents_dir = scripts_dir.parent
            project_root = agents_dir.parent
            for build_type in ["release", "debug"]:
                candidate = project_root / "target" / build_type / "robot_brain"
                if candidate.exists():
                    server_path = candidate
                    break
                exe_candidate = candidate.with_suffix(".exe")
                if exe_candidate.exists():
                    server_path = exe_candidate
                    break

        if server_path is None or not server_path.exists():
            raise FileNotFoundError(
                f"robot_brain binary not found\n"
                f"Searched:\n"
                f"  {project_root}/target/release/robot_brain\n"
                f"  {project_root}/target/release/robot_brain.exe\n"
                f"Build it: cd {project_root} && cargo build --release"
            )

        client = cls(server_path)
        await client.start()
        return client

    async def start(self) -> None:
        """Start robot_brain subprocess and complete MCP handshake."""
        self._process = await asyncio.create_subprocess_exec(
            str(self.server_path),
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        # Send initialize request (protocol version 2025-03-26, matching Rust client)
        await self._send(
            "initialize",
            {
                "protocolVersion": "2025-03-26",
                "capabilities": {"tools": {}},
                "clientInfo": {"name": "test_suite2", "version": "0.0.1"},
            },
        )
        # Send notifications/initialized (required by MCP spec)
        await self._send_notification("notifications/initialized", {})
        # Discard the initialize response
        await self._read_response_line()

    async def stop(self) -> None:
        """Stop robot_brain subprocess."""
        if self._process and self._process.returncode is None:
            self._process.terminate()
            try:
                await asyncio.wait_for(self._process.wait(), timeout=5.0)
            except asyncio.TimeoutError:
                self._process.kill()

    async def get_workflow(self, purpose: str = "general") -> dict:
        """Call get_workflow (required before other tools)."""
        result = await self._call_tool_impl("get_workflow", {"purpose": purpose})
        self._workflow_retrieved = True
        return result

    async def search_memory_first(self, query: str = "test") -> dict:
        """Call search_memory (required after get_workflow)."""
        result = await self._call_tool_impl("search_memory", {"query": query})
        self._memory_searched = True
        return result

    async def call_tool(self, tool_name: str, params: dict) -> dict:
        """Call any MCP tool."""
        if not self._workflow_retrieved:
            raise RuntimeError("Must call get_workflow() first")
        if not self._memory_searched and tool_name != "search_memory":
            raise RuntimeError("Must call search_memory() first")
        return await self._call_tool_impl(tool_name, params)

    async def list_tools(self) -> list:
        """List all available tools."""
        await self._send("tools/list", {})
        response = await self._read_response_line()
        data = json.loads(response)
        return data["result"]["tools"]

    async def _send(self, method: str, params: dict) -> None:
        """Send a JSON-RPC request (line-based)."""
        self._request_id += 1
        request = json.dumps(
            {
                "jsonrpc": "2.0",
                "id": self._request_id,
                "method": method,
                "params": params,
            }
        )
        # Write JSON followed by newline (matching Rust client protocol)
        process = self._process
        if process is None:
            raise RuntimeError("Process not started")
        process = cast(asyncio.subprocess.Process, process)
        process.stdin.write((request + "\n").encode())  # type: ignore
        await process.stdin.drain()  # type: ignore

    async def _send_notification(self, method: str, params: dict) -> None:
        """Send a notification (no response expected)."""
        notification = json.dumps(
            {
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
            }
        )
        process = self._process
        if process is None:
            raise RuntimeError("Process not started")
        process = cast(asyncio.subprocess.Process, process)
        process.stdin.write((notification + "\n").encode())  # type: ignore
        await process.stdin.drain()  # type: ignore

    async def _read_response_line(self) -> str:
        """Read one line from stdout that looks like a JSON-RPC response."""
        process = self._process
        if process is None:
            raise RuntimeError("Process not started")
        process = cast(asyncio.subprocess.Process, process)
        for _ in range(100):  # Safety limit
            line = await process.stdout.readline()  # type: ignore
            line_str = line.decode().strip()
            if line_str and line_str.startswith("{") and "jsonrpc" in line_str:
                return line_str
        raise RuntimeError("No JSON-RPC response received")

    async def _call_tool_impl(self, tool_name: str, params: dict) -> dict:
        """Call a tool and return the parsed result."""
        await self._send(
            "tools/call",
            {
                "name": tool_name,
                "arguments": params,
            },
        )
        response = await self._read_response_line()
        data = json.loads(response)
        result = data.get("result", {})
        # MCP returns content as [{"type": "text", "text": "..."}]
        # The text field may contain a JSON string that needs parsing
        content = result.get("content")
        if content:
            text = content[0].get("text", "")
            if text:
                try:
                    return json.loads(text)
                except json.JSONDecodeError:
                    pass
        return result
