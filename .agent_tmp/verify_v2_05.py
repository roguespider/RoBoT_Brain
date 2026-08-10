#!/usr/bin/env python3
"""Verify V2-05: MCP tool outcomes auto-emit experiences."""
import json, subprocess, os, select, time

ROBOT_BRAIN = "/workspace/project/RoBoT_Brain/target/release/robot_brain"

def read_response(proc, timeout=20):
    start = time.time(); buf = b""
    while True:
        ready, _, _ = select.select([proc.stdout], [], [], 1.0)
        if ready:
            chunk = os.read(proc.stdout.fileno(), 4096); buf += chunk
            while b"\n" in buf:
                line, buf = buf.split(b"\n", 1); line = line.strip()
                if line.startswith(b"{") and b'"jsonrpc"' in line:
                    try: return json.loads(line)
                    except: pass
        if time.time() - start > timeout: return None

def send(proc, method, params=None, mid=1, notify=False):
    req = {"jsonrpc": "2.0", "method": method}
    if not notify: req["id"] = mid
    if params is not None: req["params"] = params
    proc.stdin.write((json.dumps(req) + "\n").encode()); proc.stdin.flush()
    if notify: return None
    return read_response(proc)

def call_tool(proc, name, args=None, mid=1):
    resp = send(proc, "tools/call", {"name": name, "arguments": args or {}}, mid)
    if resp and "result" in resp:
        content = resp["result"].get("content", [])
        is_err = resp["result"].get("isError", False)
        text = "".join(c.get("text","") for c in content if c.get("type")=="text")
        return is_err, text
    elif resp and "error" in resp:
        return True, json.dumps(resp["error"])
    return True, str(resp)

proc = subprocess.Popen([ROBOT_BRAIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE, bufsize=0)
mid = 1
send(proc, "initialize", {"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"clientInfo":{"name":"verify","version":"1.0"}}, mid); mid+=1
send(proc, "notifications/initialized", {}, notify=True)
call_tool(proc, "get_workflow", {"purpose":"general"}, mid); mid+=1
# Workflow enforcement requires search_memory before substantive actions
call_tool(proc, "search_memory", {"query": "important memory"}, mid); mid+=1

# Test run_agent_goal — exercises the full safety gate (§16) + cognitive loop
print("=== RUN_AGENT_GOAL (exercises safety gate) ===")
is_err, resp = call_tool(proc, "run_agent_goal", {
    "goal": "Find and summarize the most important stored memory"
}, mid); mid+=1
print(f"isError: {is_err}")
print(f"RAW: {resp[:800]}")
try:
    r = json.loads(resp)
    print(f"  status: {r.get('status','')}")
    print(f"  action: {r.get('action_description','')}")
    print(f"  confidence: {r.get('confidence_value','')}")
    print(f"  abstain_reason: {r.get('abstain_reason','')}")
    print(f"  experience_id: {r.get('experience_id','')}")
except:
    print(resp[:500])

proc.terminate()
print("\n=== DONE ===")
