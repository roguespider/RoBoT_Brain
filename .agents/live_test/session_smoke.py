#!/usr/bin/env python3
"""Live MCP smoke test — proof that the agent connected to robot_brain itself.

stdlib-only (no deps). Spawns robot_brain, runs the workflow gate
(get_workflow -> search_memory), then calls store_memory with the REAL live
schema and asserts it succeeds. Prints PROOF: <result> on success.

This is the newspaper hit for "you must connect to MCP yourself".
"""
import json
import os
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CLIENT_DIR = os.path.join(REPO_ROOT, ".agents", "live_test")
sys.path.insert(0, CLIENT_DIR)

from mcp_client import RobotBrainClient  # noqa: E402


def main() -> int:
    with RobotBrainClient() as c:
        c.init()  # initialize + workflow gate (get_workflow -> search_memory)

        # store_memory: the handler requires `memory_type` (the advertised
        # inputSchema on some builds lists content/category/importance and
        # OMITS memory_type — a known schema/handler mismatch we surfaced by
        # connecting live). Call the real contract.
        store_ok = False
        store_id = ""
        try:
            r = c.call("store_memory", {
                "content": "session_start smoke: live MCP confirmed",
                "memory_type": "note",
            })
            payload = r.text_json()
            if isinstance(payload, dict):
                store_id = payload.get("id", payload.get("experience_id", ""))
                store_ok = bool(store_id) or "success" in json.dumps(payload).lower()
        except Exception as e:  # noqa: BLE001
            print(f"store_memory call failed: {e}", file=sys.stderr)
            return 2

        # search_memory must return results (proves the memory system works live).
        search_ok = False
        try:
            s = c.call("search_memory", {"query": "live MCP confirmed"})
            sp = s.text_json()
            search_ok = isinstance(sp, dict) and (
                sp.get("count", 0) > 0 or len(sp.get("results", [])) > 0
            )
        except Exception as e:  # noqa: BLE001
            print(f"search_memory call failed: {e}", file=sys.stderr)
            return 3

        # A planning tool to confirm a non-memory path works live.
        plan_ok = False
        try:
            p = c.call("create_plan", {"goal": "session smoke plan", "context": "startup"})
            pp = p.text_json()
            plan_ok = isinstance(pp, dict) and bool(pp.get("id") or pp.get("plan"))
        except Exception as e:  # noqa: BLE001
            print(f"create_plan call failed: {e}", file=sys.stderr)
            return 4

        if not (store_ok and search_ok and plan_ok):
            print(
                f"smoke incomplete: store={store_ok} search={search_ok} plan={plan_ok}",
                file=sys.stderr,
            )
            return 5

        proof = f"store_id={store_id[:8]} search=ok plan=ok"
        print(f"PROOF:{proof}")
        print(json.dumps({"store_ok": store_ok, "search_ok": search_ok, "plan_ok": plan_ok}))
        return 0


if __name__ == "__main__":
    sys.exit(main())
