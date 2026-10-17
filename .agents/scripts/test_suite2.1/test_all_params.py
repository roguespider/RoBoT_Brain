#!/usr/bin/env python3
"""
test_all_params.py — Every tool from all_tools_list.txt with parameter testing.
Tests robot_brain.exe as MCP: all 206 registered tools with full/default params.
"""

import sys
from pathlib import Path

import pytest

_test_dir = Path(__file__).resolve().parent
if str(_test_dir) not in sys.path:
    sys.path.insert(0, str(_test_dir))

from mcp_client import RobotBrainClient

FULL_PARAM_DEFAULTS = {
    "abandon_exploration": {"exploration_id": "param-id"},
    "acp_agent_count": {},
    "acp_registry": {},
    "acp_router": {},
    "add_evidence": {
        "hypothesis_id": "param-id",
        "content": "param evidence",
        "evidence_type": "success",
        "direction": "support",
        "strength": 0.5,
    },
    "add_evidence_to_experience": {
        "experience_id": "param-id",
        "evidence_id": "param-id",
    },
    "add_hypothesis": {
        "exploration_id": "param-id",
        "statement": "param hypothesis",
        "initial_confidence": 0.5,
    },
    "add_knowledge": {
        "statement": "param knowledge",
        "knowledge_type": "fact",
        "tags": ["param"],
        "confidence": 0.8,
    },
    "add_knowledge_relation": {
        "knowledge_id": "param-id",
        "related_id": "param-id",
        "relation_type": "related",
        "confidence": 0.7,
    },
    "add_plan_step": {
        "plan_id": "param-id",
        "description": "param step",
        "action": "param action",
    },
    "add_step_dependency": {
        "plan_id": "param-id",
        "step_id": "param-step",
        "depends_on": "param-dep",
    },
    "add_workflow_step": {
        "workflow_id": "param-id",
        "name": "param step",
        "action": "param action",
        "parameters": "{}",
    },
    "add_world_relationship": {
        "source_id": "param-src",
        "target_id": "param-tgt",
        "kind": "related_to",
        "confidence": 0.6,
    },
    "analyze_patterns": {"lookback_days": 7},
    "apply_personality_preset": {"preset": "balanced"},
    "apply_skill_decay": {},
    "archive_experience": {"experience_id": "param-id"},
    "archive_memory": {"memory_id": "param-id"},
    "bump_knowledge_version": {
        "knowledge_id": "param-id",
        "bump_type": "minor",
        "initial_version": "1.0.0",
    },
    "call_tool": {"tool_name": "list_tools", "arguments": {}},
    "cancel_plan": {"plan_id": "param-id"},
    "cancel_workflow": {"workflow_id": "param-id"},
    "cleanup_sessions": {},
    "clear_skill_metrics": {"skill_id": "param-id"},
    "complete_exploration": {
        "exploration_id": "param-id",
        "findings": [{"description": "param finding", "confidence": 0.85}],
    },
    "complete_step": {
        "plan_id": "param-id",
        "step_id": "param-step",
        "outcome": "completed",
    },
    "connect_mcp_server": {
        "name": "test-server",
        "command": ["echo", "test"],
        "args": ["--test"],
    },
    "cooboploop_add_strategic_objective": {
        "name": "param-obj",
        "category": "param-cat",
    },
    "cooboploop_configure_idle_reevaluation_interval": {"seconds": 300},
    "cooboploop_create_research_objective": {
        "topic": "param topic",
        "priority": 1.0,
        "persistence_target": "memory",
    },
    "cooboploop_detect_hardware_changes": {},
    "cooboploop_enqueue_goal": {
        "title": "param goal",
        "description": "param desc",
        "deadline": "2026-12-31T23:59:59Z",
        "dependencies": ["param-dep"],
        "expected_value": 10.0,
        "learning_value": 5.0,
        "required_capabilities": ["param-cap"],
        "risk": 0.3,
        "source": "param-source",
    },
    "cooboploop_evaluate_goal": {"goal_id": "param-goal-id"},
    "cooboploop_get_autonomous_mode": {},
    "cooboploop_get_autonomy_levels": {},
    "cooboploop_get_capability_assessment": {"capability_id": "param-cap-id"},
    "cooboploop_get_goal": {"goal_id": "param-goal-id"},
    "cooboploop_get_hardware_profile": {},
    "cooboploop_get_idle_state": {},
    "cooboploop_get_loop_status": {},
    "cooboploop_get_modification_boundary": {},
    "cooboploop_get_objective_hierarchy": {},
    "cooboploop_get_pending_external_opportunities": {},
    "cooboploop_list_capabilities": {},
    "cooboploop_list_goals": {"limit": 10, "offset": 0, "status_filter": ""},
    "cooboploop_list_strategic_objectives": {},
    "cooboploop_promote_autonomy": {"capability_id": "param-cap-id"},
    "cooboploop_record_capability_outcome": {
        "capability_id": "param-cap-id",
        "success": True,
    },
    "cooboploop_remove_strategic_objective": {"id": "param-obj-id"},
    "cooboploop_reprioritize_queue": {},
    "cooboploop_run_inspection": {"target": "database"},
    "cooboploop_run_opportunity_intake": {
        "source_url": "https://example.com/feed",
        "source_type": "rss",
    },
    "cooboploop_run_post_task_evaluation": {"goal_id": "param-goal-id"},
    "cooboploop_run_single_cycle": {},
    "cooboploop_run_source_discovery": {"source_type": "rss"},
    "cooboploop_set_autonomous_mode": {"enabled": True},
    "cooboploop_set_mission": {"mission": "param mission statement"},
    "cooboploop_set_modification_boundary": {"boundary": "strict"},
    "cooboploop_set_priority_policy": {"policy": "conservative"},
    "cooboploop_start_loop": {"max_cycles": 10},
    "cooboploop_step_loop": {},
    "cooboploop_stop_loop": {},
    "cooboploop_update_goal_status": {
        "goal_id": "param-goal-id",
        "new_status": "Active",
    },
    "create_acp_message": {
        "message_type": "Inform",
        "payload": {"test": True},
        "receiver": {"agent_type": "test", "instance_id": "test"},
        "sender": {"agent_type": "system", "instance_id": "main"},
        "ttl": 60,
    },
    "create_hypothesis": {
        "statement": "param hypothesis statement",
        "domain": "param-domain",
        "source_observations": ["obs-1", "obs-2"],
    },
    "create_plan": {"goal": "param goal description"},
    "create_reflection": {
        "title": "param reflection title",
        "content": "param reflection content",
        "experience_ids": ["exp-1"],
    },
    "create_workflow": {"name": "param workflow name"},
    "deep_research": {"query": "param deep research query"},
    "delete_embedding": {"memory_id": "param-memory-id"},
    "delete_ingested_files": {"files": ["param/file.txt"], "confirmation": "yes"},
    "delete_knowledge": {"knowledge_id": "param-knowledge-id"},
    "delete_memory_by_id": {"memory_id": "param-memory-id"},
    "delete_workflow": {"workflow_id": "param-workflow-id"},
    "detect_trends": {"platforms": ["reddit", "bluesky", "mastodon"]},
    "discover_skill": {
        "name": "param-skill-name",
        "description": "param description",
        "category": "param-category",
    },
    "enable_disable_skill": {"skill_id": "param-skill-id", "enabled": False},
    "evaluate_exploration_hypothesis": {
        "exploration_id": "param-expl-id",
        "hypothesis_id": "param-hyp-id",
        "result": "supported",
    },
    "evaluate_hypothesis": {"hypothesis_id": "param-hyp-id"},
    "execute_skill": {"skill_id": "param-skill-id", "parameters": "{}"},
    "extract_knowledge": {
        "hypothesis_id": "param-hyp-id",
        "knowledge_content": "param extracted content",
    },
    "fail_step": {
        "plan_id": "param-plan-id",
        "step_id": "param-step-id",
        "reason": "param failure reason",
    },
    "find_counter_arguments": {"query": "param query for counter args"},
    "find_error_resolution": {"error": "param error message"},
    "find_world_entity": {"name": "param entity name"},
    "format_citations": {"doi": "10.1000/param-doi", "format": "APA"},
    "format_response": {"content": "param content to format", "style": "balanced"},
    "get_agent_capabilities": {"agent_id": "param-agent-id"},
    "get_consumed_resources": {"id": "param-resource-id"},
    "get_embedding": {"memory_id": "param-memory-id"},
    "get_embedding_stats": {},
    "get_evidence": {"evidence_id": "param-evidence-id"},
    "get_experience": {"id": "param-experience-id"},
    "get_experience_stats": {"period": "week"},
    "get_exploration_status": {"exploration_id": "param-expl-id"},
    "get_hypothesis": {"hypothesis_id": "param-hyp-id"},
    "get_insights": {"domain": "param-domain", "limit": 5},
    "get_knowledge": {"domain": "param-domain", "limit": 5},
    "get_knowledge_stats": {},
    "get_mature_knowledge": {"limit": 5},
    "get_memory": {"id": "param-memory-id"},
    "get_patterns": {"pattern_type": "success"},
    "get_personality": {},
    "get_personality_decision": {
        "confidence": 0.7,
        "potential_gain": 10,
        "potential_loss": 2,
        "uncertainty": 0.3,
    },
    "get_plan": {"plan_id": "param-plan-id"},
    "get_recommendations": {"category": "param-category"},
    "get_related_knowledge": {"knowledge_id": "param-knowledge-id"},
    "get_reputation": {"tool_name": "param-tool-name"},
    "get_session_state": {},
    "get_skill": {"skill_id": "param-skill-id"},
    "get_skill_metrics": {"skill_id": "param-skill-id"},
    "get_skill_recommendations": {"task": "param-task"},
    "get_skill_stats": {},
    "get_system_status": {},
    "get_tool": {"name": "store_memory"},
    "get_unreliable_skills": {},
    "get_wikipedia": {"query": "param wikipedia query"},
    "get_worker_count": {},
    "get_worker_stats": {"observer_name": "param-observer"},
    "get_workflow": {"purpose": "general"},
    "get_workflow_status": {"workflow_id": "param-workflow-id"},
    "get_world_blockers": {"id": "param-id"},
    "get_world_dependencies": {"id": "param-id"},
    "get_world_entity": {"id": "param-id"},
    "get_world_model_stats": {},
    "get_world_relationships": {"id": "param-id"},
    "global_search": {"query": "param global query", "limit": 10},
    "ingest_files": {"folder": "files_to_import", "limit": 1, "memory_type": "file"},
    "link_memories": {"from_id": "param-from", "to_id": "param-to"},
    "list_acp_agents": {},
    "list_embeddings": {"limit": 5},
    "list_evidence": {"direction": "support", "evidence_type": "success", "limit": 5},
    "list_experiences": {"experience_type": "task", "limit": 5},
    "list_hypotheses": {"domain": "param", "limit": 5},
    "list_importable": {"folder": "files_to_import", "limit": 10},
    "list_ingested_files": {"folder": "files_to_import", "limit": 10},
    "list_memories": {"limit": 5},
    "list_observations": {"limit": 5, "observation_type": "pattern"},
    "list_personality_presets": {},
    "list_plans": {"status": "pending"},
    "list_reflections_by_status": {"status": "draft"},
    "list_skills": {"enabled_only": False},
    "list_tools": {},
    "list_workflows": {},
    "list_world_entities": {"kind": "object"},
    "manage_knowledge_dependency": {
        "knowledge_id": "param-id",
        "action": "get",
        "dependency_type": "required",
        "depends_on_id": "param-dep-id",
    },
    "pause_exploration": {"exploration_id": "param-id"},
    "pause_workflow": {"workflow_id": "param-id"},
    "promote_finding": {"exploration_id": "param-id", "finding_id": "param-id"},
    "query_knowledge": {
        "query": "param",
        "knowledge_type": "fact",
        "min_confidence": 0.5,
        "limit": 5,
    },
    "quick_research": {"query": "param quick query"},
    "ranked_search": {"query": "param", "limit": 5},
    "record_attempt": {
        "exploration_id": "param-id",
        "action": "param action",
        "expected_result": "param expected",
        "actual_result": "param actual",
    },
    "record_experience": {
        "title": "param exp",
        "description": "param desc",
        "experience_type": "task",
        "outcome": "success",
        "context": "{}",
    },
    "record_knowledge_application": {
        "knowledge_id": "param-id",
        "success": True,
        "context": "param context",
    },
    "record_observation": {
        "content": "param observation",
        "context": "param context",
        "observation_type": "pattern",
    },
    "register_agent": {
        "agent_type": "param-agent",
        "instance_id": "param-id",
        "capabilities": ["param-cap"],
    },
    "register_skill": {
        "name": "param-skill",
        "description": "param desc",
        "category": "param-cat",
        "parameters": "{}",
    },
    "research": {"query": "param research query"},
    "resume_exploration": {"exploration_id": "param-id"},
    "resume_workflow": {"workflow_id": "param-id"},
    "resurrect_dead_link": {"url": "https://example.com/dead"},
    "route_acp_message": {
        "message_type": "Inform",
        "payload": {"test": True},
        "receiver": {"agent_type": "param", "instance_id": "param"},
        "sender": {"agent_type": "system", "instance_id": "param"},
        "ttl": 60,
    },
    "run_agent_goal": {"goal": "param agent goal", "confidence_threshold": 0.5},
    "score_reliability": {"urls": ["https://example.com"]},
    "search_bluesky": {"query": "param", "sort": "top"},
    "search_datasets": {"query": "param dataset"},
    "search_hackernews": {"query": "param hn"},
    "search_knowledge_by_tag": {"tag": "param"},
    "search_mastodon": {"query": "param"},
    "search_memory": {"query": "param", "limit": 5},
    "search_news": {"query": "param news"},
    "search_osm": {"query": "param osm", "location": "param loc"},
    "search_preprints": {"query": "param preprint"},
    "search_reddit": {"query": "param reddit", "subreddit": "param-sub"},
    "search_sec_filings": {"query": "param sec", "filing_type": "10-K"},
    "search_similar": {
        "query_embedding": [0.1, 0.2, 0.3],
        "min_similarity": 0.5,
        "limit": 5,
    },
    "search_skills": {"query": "param"},
    "search_skills_by_tag": {"tag": "param"},
    "search_substack": {"publications": "param-pub", "max_posts": 5},
    "search_telegram": {"channel": "param-channel", "max_messages": 5},
    "search_vk": {"query": "param vk"},
    "search_youtube": {"query": "param youtube", "include_transcript": False},
    "set_knowledge_status": {"knowledge_id": "param-id", "action": "activate"},
    "set_personality_traits": {
        "curiosity": 0.5,
        "caution": 0.5,
        "creativity": 0.5,
        "patience": 0.5,
        "thoroughness": 0.5,
        "verbosity": 0.5,
        "risk_tolerance": 0.5,
        "humor_level": 0.5,
    },
    "set_workflow_variable": {
        "workflow_id": "param-id",
        "key": "param-key",
        "value": "param-value",
    },
    "start_exploration": {"title": "param exploration", "purpose": "param purpose"},
    "start_plan": {"plan_id": "param-id"},
    "start_workflow": {"workflow_id": "param-id"},
    "store_embedding": {"memory_id": "param-id", "embedding": [0.1, 0.2, 0.3]},
    "store_memory": {
        "content": "param memory content",
        "memory_type": "fact",
        "tags": ["param"],
        "confidence": 0.8,
        "importance": 5.0,
    },
    "transcribe_audio": {"path": "param.wav", "store_as_memory": False},
    "unregister_agent": {"agent_type": "param-agent", "instance_id": "param-id"},
    "unregister_skill": {"skill_id": "param-skill-id"},
    "update_knowledge": {
        "knowledge_id": "param-id",
        "statement": "updated statement",
        "confidence": 0.9,
        "tags": ["updated"],
    },
    "update_reflection": {
        "reflection_id": "param-id",
        "title": "updated title",
        "description": "updated desc",
        "summary": "updated summary",
    },
    "update_skill_mastery": {"skill_id": "param-id", "mastery": 0.75, "success": True},
    "upsert_world_entity": {
        "name": "param entity",
        "kind": "object",
        "confidence": 0.8,
        "properties": {"test": True},
    },
    "validate_bibliography": {
        "bibliography": [
            {"title": "param title", "author": "param author", "year": 2024}
        ]
    },
    "validate_knowledge_dependencies": {},
    "validate_reflection": {"reflection_id": "param-id"},
    "verify_citations": {"references": [{"title": "param", "doi": "10.1000/param"}]},
    "web_extract": {"url": "https://example.com"},
    "web_open": {"url": "https://example.com"},
    "web_search": {"query": "param web search"},
}


@pytest.mark.asyncio
async def test_all_206_tools_with_params(client):
    """Every registered tool called with its full/default parameter set."""
    # Load full catalog
    tool_list_path = Path(__file__).resolve().parent / "all_tools_list.txt"
    if not tool_list_path.exists():
        pytest.skip("all_tools_list.txt not found")
    tool_names = [
        line.strip() for line in tool_list_path.read_text().splitlines() if line.strip()
    ]

    # Cross-check with live server
    live_tools = await client.list_tools()
    live_names = {t.get("name", "") for t in live_tools}

    tested = 0
    failed = []

    for name in tool_names:
        params = FULL_PARAM_DEFAULTS.get(name, {})
        try:
            await client.call_tool(name, params)
            tested += 1
        except Exception as e:
            # Document failure but don't fail the suite — this is a parameter coverage test
            failed.append((name, str(e)[:120]))

    # Gate: must have tested all advertised tools; failures documented
    assert tested == len(tool_names), (
        f"Only {tested}/{len(tool_names)} tools responded; missing: {set(tool_names) - live_names}"
    )
    # Print failures for triage (not gate-failing — master gate handles red/green)
    for name, err in failed:
        print(f"[FAIL] {name}: {err}")
