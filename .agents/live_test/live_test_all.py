#!/usr/bin/env python3
"""Comprehensive live test for robot_brain MCP + ACP tools.

Connects to the compiled ``robot_brain`` binary over stdio MCP
(protocol 2025-03-26) and exercises every tool category to confirm
they function correctly in a live environment.

Run::

    python .agents/live_test/live_test_all.py

Exit code 0 = all tested tools passed; 1 = one or more failures.
A summary table is printed at the end.
"""

from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mcp_client import RobotBrainClient, default_binary


def clean_db():
    d = os.path.dirname(default_binary())
    for name in ("robot_brain.db",):
        p = os.path.join(d, name)
        if os.path.exists(p):
            os.remove(p)


class Reporter:
    def __init__(self):
        self.results: list[tuple[str, str, str]] = []  # (category, tool, status)

    def record(self, category: str, tool: str, ok: bool, detail: str = ""):
        status = "OK" if ok else "FAIL"
        self.results.append((category, tool, status))
        suffix = f"  {detail}" if detail else ""
        print(f"  [{category}] {tool:35s} {status}{suffix}")

    def summary(self) -> int:
        total = len(self.results)
        ok = sum(1 for _, _, s in self.results if s == "OK")
        fails = [(c, t) for c, t, s in self.results if s == "FAIL"]
        print("-" * 64)
        print(f"=== Live test summary: {ok}/{total} tools OK ===")
        if fails:
            print("FAILURES:")
            for c, t in fails:
                print(f"  [{c}] {t}")
            return 1
        print("ALL TESTED TOOLS PASSED")
        return 0


def truthy(val) -> bool:
    """A tool result is considered successful if it is not an error and
    contains some truthy indicator (id/count/registered/success) or at
    least returned non-error JSON."""
    if val is None:
        return False
    if isinstance(val, bool):
        return val
    if isinstance(val, (int, float)):
        return val >= 0
    if isinstance(val, dict):
        if val.get("error"):
            return False
        if val.get("success") is False:
            return False
        return True
    return bool(val)


def main():
    clean_db()
    binary = default_binary()
    print(f"=== robot_brain comprehensive live test ===")
    print(f"    binary: {binary}")
    print(f"    protocol: 2025-03-26\n")
    r = Reporter()

    with RobotBrainClient() as c:
        init = c.init()
        proto = init.get("result", {}).get("protocolVersion", "?")
        if proto != "2025-03-26":
            r.record("protocol", "initialize", False, f"negotiated={proto}")
        else:
            r.record("protocol", "initialize", True, f"negotiated={proto}")

        tools = c.list_tools()
        r.record("discovery", "list_tools", len(tools) > 50, f"{len(tools)} tools")

        # ---- ACP tools (thorough) ----
        print("\n-- ACP --")
        # registry info (no args)
        for tname in ("acp_agent_count", "acp_registry", "acp_router", "list_acp_agents", "get_system_status"):
            res = c.call(tname, {})
            r.record("acp", tname, not res.is_error, str(res.text_json())[:60])

        # register an agent
        res = c.call("register_agent", {
            "agent_type": "researcher",
            "instance_id": "live-1",
            "capabilities": ["analysis", "verification"],
        })
        reg = res.text_json()
        r.record("acp", "register_agent", truthy(reg.get("registered")), f"registered={reg.get('registered')}")

        # verify it shows up
        res = c.call("acp_agent_count", {})
        cnt = res.text_json()
        ac = cnt.get("count", cnt.get("agent_count", 0)) if isinstance(cnt, dict) else 0
        r.record("acp", "acp_agent_count(after register)", ac >= 1, f"count={ac}")

        res = c.call("list_acp_agents", {})
        agents = res.text_json()
        r.record("acp", "list_acp_agents(after register)", not res.is_error)

        # get_agent_capabilities for the registered agent
        res = c.call("get_agent_capabilities", {"agent_id": "researcher:live-1"})
        caps = res.text_json()
        r.record("acp", "get_agent_capabilities", not res.is_error, str(caps)[:60])

        # create_acp_message (no routing)
        res = c.call("create_acp_message", {
            "sender": {"agent_type": "researcher", "instance_id": "live-1"},
            "receiver": {"agent_type": "researcher", "instance_id": "live-1"},
            "message_type": "ping",
            "payload": {"hello": "world"},
        })
        msg = res.text_json()
        r.record("acp", "create_acp_message", not res.is_error, str(msg)[:60])

        # route_acp_message
        res = c.call("route_acp_message", {
            "sender": {"agent_type": "researcher", "instance_id": "live-1"},
            "receiver": {"agent_type": "researcher", "instance_id": "live-1"},
            "message_type": "ping",
            "payload": {"route": True},
        })
        routed = res.text_json()
        r.record("acp", "route_acp_message", not res.is_error, str(routed)[:60])

        # unregister the agent
        res = c.call("unregister_agent", {
            "agent_type": "researcher",
            "instance_id": "live-1",
        })
        unreg = res.text_json()
        r.record("acp", "unregister_agent", truthy(unreg.get("unregistered")), f"unregistered={unreg.get('unregistered')}")

        # ---- Memory tools ----
        print("\n-- Memory --")
        res = c.call("store_memory", {"content": "Live ACP/MCP verification memory", "memory_type": "note"})
        sm = res.text_json()
        mid = sm.get("memory_id") or sm.get("id")
        r.record("memory", "store_memory", bool(mid), f"id={mid}")

        res = c.call("search_memory", {"query": "verification memory"})
        sres = res.text_json()
        r.record("memory", "search_memory", not res.is_error, f"count={sres.get('count','?')}")

        if mid:
            res = c.call("get_memory", {"id": mid})
            r.record("memory", "get_memory", not res.is_error)

        res = c.call("list_memories", {"limit": 5})
        r.record("memory", "list_memories", not res.is_error)

        if mid:
            res = c.call("archive_memory", {"memory_id": mid})
            r.record("memory", "archive_memory", not res.is_error, str(res.text_json())[:50])

        # ---- Experience tools ----
        print("\n-- Experience --")
        res = c.call("record_experience", {
            "title": "Live verification",
            "description": "Exercised MCP and ACP tools live",
            "experience_type": "verification",
            "outcome": "Success",
            "context": "{}",
        })
        exp = res.text_json()
        eid = exp.get("experience_id") or exp.get("id")
        r.record("experience", "record_experience", bool(eid), f"id={eid}")

        res = c.call("list_experiences", {"limit": 5})
        r.record("experience", "list_experiences", not res.is_error)

        res = c.call("get_experience_stats", {})
        r.record("experience", "get_experience_stats", not res.is_error)

        res = c.call("get_insights", {})
        r.record("experience", "get_insights", not res.is_error)

        res = c.call("analyze_patterns", {})
        r.record("experience", "analyze_patterns", not res.is_error)

        res = c.call("get_patterns", {})
        r.record("experience", "get_patterns", not res.is_error)

        if eid:
            res = c.call("get_experience", {"id": eid})
            r.record("experience", "get_experience", not res.is_error)

        # ---- Knowledge tools ----
        print("\n-- Knowledge --")
        res = c.call("add_knowledge", {
            "statement": "MCP protocol 2025-03-26 is negotiated correctly by robot_brain",
            "knowledge_type": "fact",
            "confidence": 0.9,
            "source": "live-test",
            "tags": ["mcp", "protocol"],
        })
        ak = res.text_json()
        kid = ak.get("knowledge_id") or ak.get("id")
        r.record("knowledge", "add_knowledge", bool(kid) or not res.is_error, f"id={kid}")

        res = c.call("query_knowledge", {"query": "MCP protocol"})
        r.record("knowledge", "query_knowledge", not res.is_error)

        res = c.call("get_knowledge", {})
        r.record("knowledge", "get_knowledge", not res.is_error)

        res = c.call("get_knowledge_stats", {})
        r.record("knowledge", "get_knowledge_stats", not res.is_error)

        res = c.call("global_search", {"query": "protocol"})
        r.record("knowledge", "global_search", not res.is_error)

        # ---- Planning tools ----
        print("\n-- Planning --")
        res = c.call("create_plan", {"goal": "Verify all tool categories live"})
        cp = res.text_json()
        pid = cp.get("plan_id") or cp.get("id")
        r.record("planning", "create_plan", bool(pid), f"id={pid}")

        res = c.call("list_plans", {})
        r.record("planning", "list_plans", not res.is_error)

        if pid:
            res = c.call("get_plan", {"plan_id": pid})
            r.record("planning", "get_plan", not res.is_error)

            res = c.call("add_plan_step", {"plan_id": pid, "description": "Test ACP", "action": "verify_acp"})
            r.record("planning", "add_plan_step", not res.is_error, str(res.text_json())[:50])

            res = c.call("start_plan", {"plan_id": pid})
            r.record("planning", "start_plan", not res.is_error, str(res.text_json())[:50])

        # ---- Workflow tools ----
        print("\n-- Workflows --")
        res = c.call("get_workflow", {"purpose": "general"})
        r.record("workflow", "get_workflow", not res.is_error)

        res = c.call("create_workflow", {"name": "live-test-wf"})
        wf = res.text_json()
        wid = wf.get("workflow_id") or wf.get("id")
        r.record("workflow", "create_workflow", bool(wid) or not res.is_error, f"id={wid}")

        res = c.call("list_workflows", {})
        r.record("workflow", "list_workflows", not res.is_error)

        # ---- Hypothesis tools ----
        print("\n-- Hypothesis --")
        res = c.call("create_hypothesis", {
            "statement": "ACP message routing delivers messages to registered agents",
            "domain": "acp",
            "source_observations": [],
        })
        hyp = res.text_json()
        if isinstance(hyp, dict):
            inner = hyp.get("hypothesis", hyp)
            hid = inner.get("id") or hyp.get("hypothesis_id") or hyp.get("id")
        else:
            hid = None
        r.record("hypothesis", "create_hypothesis", bool(hid) or not res.is_error, f"id={hid}")

        if hid:
            res = c.call("add_evidence", {
                "hypothesis_id": hid,
                "content": "route_acp_message returned without error",
                "evidence_type": "success",
                "direction": "support",
                "strength": 0.8,
            })
            r.record("hypothesis", "add_evidence", not res.is_error)

            res = c.call("evaluate_hypothesis", {"hypothesis_id": hid})
            r.record("hypothesis", "evaluate_hypothesis", not res.is_error)

        res = c.call("list_hypotheses", {})
        r.record("hypothesis", "list_hypotheses", not res.is_error)

        # ---- Exploration tools ----
        print("\n-- Exploration --")
        res = c.call("start_exploration", {"title": "MCP protocol compliance", "purpose": "verify tools live"})
        expl = res.text_json()
        if isinstance(expl, dict):
            eid2 = expl.get("exploration_id") or expl.get("id")
        else:
            eid2 = None
        r.record("exploration", "start_exploration", bool(eid2) or not res.is_error, f"id={eid2}")

        if eid2:
            res = c.call("get_exploration_status", {"exploration_id": eid2})
            r.record("exploration", "get_exploration_status", not res.is_error)

            res = c.call("record_attempt", {
                "exploration_id": eid2,
                "action": "test_acp_routing",
                "expected_result": "message routed",
                "actual_result": "message routed successfully",
            })
            r.record("exploration", "record_attempt", not res.is_error)

            res = c.call("add_hypothesis", {
                "exploration_id": eid2,
                "statement": "ACP routing works end to end",
            })
            r.record("exploration", "add_hypothesis", not res.is_error)

        # ---- Skills tools ----
        print("\n-- Skills --")
        res = c.call("register_skill", {
            "name": "live-test-skill",
            "description": "A skill registered during live testing",
            "category": "system",
        })
        sk = res.text_json()
        if isinstance(sk, dict):
            sid = sk.get("skill_id") or sk.get("id")
        else:
            sid = None
        r.record("skills", "register_skill", bool(sid) or not res.is_error, f"id={sid}")

        res = c.call("list_skills", {})
        r.record("skills", "list_skills", not res.is_error)

        res = c.call("search_skills", {"query": "test"})
        r.record("skills", "search_skills", not res.is_error)

        res = c.call("get_skill_stats", {})
        r.record("skills", "get_skill_stats", not res.is_error)

        res = c.call("get_skill_recommendations", {"task": "testing"})
        r.record("skills", "get_skill_recommendations", not res.is_error)

        # ---- Observations ----
        print("\n-- Observations --")
        res = c.call("record_observation", {
            "content": "All MCP and ACP tools responded during live testing",
            "context": "live-verification",
            "observation_type": "system",
        })
        r.record("observation", "record_observation", not res.is_error)

        res = c.call("list_observations", {"limit": 5})
        r.record("observation", "list_observations", not res.is_error)

    print()
    return r.summary()


if __name__ == "__main__":
    sys.exit(main())
