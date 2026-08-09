"""Reusable raw JSON-RPC stdio MCP client for the robot_brain server.

Spawns the compiled ``robot_brain`` binary, negotiates MCP protocol
``2025-03-26`` (the version the server advertises), and satisfies the
server's mandated workflow gate (``get_workflow`` then ``search_memory``)
so subsequent tool calls are accepted.

Usage::

    from mcp_client import RobotBrainClient
    with RobotBrainClient() as c:
        c.init()
        r = c.call("store_memory", {"content": "hello", "memory_type": "note"})
        print(r.text_json())

This module is intentionally dependency-free (stdlib only) so it runs in
any Python environment without installing packages.
"""

from __future__ import annotations

import json
import os
import subprocess
import threading
import time
from typing import Any


def default_binary() -> str:
    env = os.environ.get("ROBOT_BRAIN_PATH")
    if env and os.path.isfile(env):
        return env
    here = os.path.dirname(os.path.abspath(__file__))
    candidates = [
        os.path.join(here, "..", "..", "target", "release", "robot_brain"),
        "/workspace/project/RoBoT_Brain/target/release/robot_brain",
    ]
    for c in candidates:
        if os.path.isfile(c):
            return os.path.normpath(c)
    return "robot_brain"


class ToolResult:
    """Wraps a ``tools/call`` JSON-RPC response."""

    def __init__(self, payload: dict | None):
        self._payload = payload or {}

    @property
    def is_error(self) -> bool:
        if self._payload.get("result", {}).get("isError"):
            return True
        parsed = self.text_json()
        return isinstance(parsed, dict) and "error" in parsed

    @property
    def content(self) -> list[dict]:
        return self._payload.get("result", {}).get("content", [])

    @property
    def text(self) -> str:
        parts = []
        for block in self.content:
            if block.get("type") == "text":
                parts.append(block.get("text", ""))
        return "\n".join(parts)

    def text_json(self) -> Any:
        t = self.text
        if not t:
            return {}
        try:
            return json.loads(t)
        except Exception:
            return t

    @property
    def raw(self) -> dict:
        return self._payload


class RobotBrainClient:
    """Live MCP client over stdio for the robot_brain binary."""

    PROTO = "2025-03-26"

    def __init__(self, binary: str | None = None, db_dir: str | None = None):
        self.binary = binary or default_binary()
        self.db_dir = db_dir or os.path.dirname(self.binary)
        self.proc: subprocess.Popen | None = None
        self._id = 0
        self._stderr_thread: threading.Thread | None = None

    def __enter__(self) -> "RobotBrainClient":
        self.start()
        return self

    def __exit__(self, *exc):
        self.close()
        return False

    def start(self):
        env = dict(os.environ)
        env["RUST_LOG"] = env.get("RUST_LOG", "")
        self.proc = subprocess.Popen(
            [self.binary],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            bufsize=0,
            cwd=self.db_dir,
            env=env,
        )
        self._stderr_thread = threading.Thread(target=self._drain_stderr, daemon=True)
        self._stderr_thread.start()

    def _drain_stderr(self):
        assert self.proc is not None
        for _line in iter(self.proc.stderr.readline, b""):
            pass

    def _send(self, method: str, params: dict) -> int:
        assert self.proc is not None and self.proc.stdin is not None
        self._id += 1
        req = {"jsonrpc": "2.0", "id": self._id, "method": method, "params": params}
        data = (json.dumps(req) + "\n").encode()
        self.proc.stdin.write(data)
        self.proc.stdin.flush()
        return self._id

    def _read_line(self, timeout: float = 20.0) -> str | None:
        assert self.proc is not None and self.proc.stdout is not None
        start = time.time()
        buf = b""
        while time.time() - start < timeout:
            byte = self.proc.stdout.read(1)
            if byte == b"":
                time.sleep(0.01)
                continue
            if byte == b"\n":
                line = buf.decode(errors="replace").strip()
                if line.startswith("{") and '"jsonrpc"' in line:
                    return line
                buf = b""
                continue
            buf += byte
        return None

    def init(self) -> dict:
        """Perform the MCP initialize handshake + workflow gate.

        Returns the raw initialize result dict.
        """
        self._send("initialize", {
            "protocolVersion": self.PROTO,
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "live_probe", "version": "1.0.0"},
        })
        resp = self._read_line(15)
        if not resp:
            raise RuntimeError("No initialize response from server")
        assert self.proc is not None and self.proc.stdin is not None
        self.proc.stdin.write(b'{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}\n')
        self.proc.stdin.flush()
        result = json.loads(resp)
        # Server enforces: get_workflow first, then search_memory before writes.
        self.call("get_workflow", {"purpose": "general"})
        self.call("search_memory", {"query": "verification probe"})
        return result

    def call(self, name: str, args: dict | None = None) -> ToolResult:
        self._send("tools/call", {"name": name, "arguments": args or {}})
        resp = self._read_line(20)
        return ToolResult(json.loads(resp) if resp else None)

    def list_tools(self) -> list[dict]:
        self._send("tools/list", {})
        resp = self._read_line(15)
        if not resp:
            return []
        return json.loads(resp).get("result", {}).get("tools", [])

    def close(self):
        if self.proc is None:
            return
        try:
            if self.proc.stdin:
                self.proc.stdin.close()
        except Exception:
            pass
        try:
            self.proc.terminate()
            self.proc.wait(timeout=5)
        except Exception:
            try:
                self.proc.kill()
            except Exception:
                pass
        self.proc = None
