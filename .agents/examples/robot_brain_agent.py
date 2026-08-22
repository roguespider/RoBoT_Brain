#!/usr/bin/env python3
"""
AI Agent connected to RoBoT Brain via MCP

This example demonstrates how to configure an AI agent (via OpenHands SDK)
to use RoBoT Brain as an MCP tool server, giving it access to:
- Memory & knowledge management
- Experience tracking & learning
- Planning & workflow execution
- Hypothesis testing & exploration
- Skill registry & execution
- ACP message routing

Usage:
    export LLM_API_KEY="your-api-key"
    export LLM_MODEL="anthropic/claude-sonnet-4-5-20250929"
    python examples/robot_brain_agent.py
"""

import os
import sys
from pathlib import Path
from typing import Optional

from pydantic import SecretStr

# Add project root to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

from openhands.sdk import (
    LLM,
    Agent,
    Conversation,
    Event,
    LLMConvertibleEvent,
    get_logger,
)
from openhands.sdk.security.llm_analyzer import LLMSecurityAnalyzer
from openhands.sdk.tool import Tool
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool

logger = get_logger(__name__)


def get_robot_brain_path() -> str:
    """Get the path to the robot_brain binary."""
    # Check environment variable first
    if os.getenv("ROBOT_BRAIN_PATH"):
        return os.getenv("ROBOT_BRAIN_PATH")

    # Check common locations
    common_paths = [
        project_root / "target" / "release" / "robot_brain",
        project_root / "target" / "debug" / "robot_brain",
        Path.home() / ".cargo" / "bin" / "robot_brain",
        Path("/usr/local/bin/robot_brain"),
    ]

    for path in common_paths:
        if path.exists():
            return str(path)

    # Default to cargo run
    return "cargo run --release -p robot_brain"


def create_robot_brain_agent(
    llm_api_key: Optional[str] = None,
    llm_model: Optional[str] = None,
    llm_base_url: Optional[str] = None,
    robot_brain_path: Optional[str] = None,
    filter_tools_regex: Optional[str] = None,
) -> Agent:
    """
    Create an AI agent configured to use RoBoT Brain MCP server.

    Args:
        llm_api_key: API key for LLM (defaults to LLM_API_KEY env var)
        llm_model: Model name (defaults to LLM_MODEL env var)
        llm_base_url: Base URL for LLM API (optional)
        robot_brain_path: Path to robot_brain binary (auto-detected if not provided)
        filter_tools_regex: Regex to filter which MCP tools are available

    Returns:
        Configured Agent instance
    """
    # Load from environment if not provided
    api_key = llm_api_key or os.getenv("LLM_API_KEY")
    if not api_key:
        raise ValueError("LLM_API_KEY environment variable is required")

    model = llm_model or os.getenv("LLM_MODEL", "anthropic/claude-sonnet-4-5-20250929")
    base_url = llm_base_url or os.getenv("LLM_BASE_URL")

    # Configure LLM
    llm = LLM(
        usage_id="robot_brain_agent",
        model=model,
        base_url=base_url,
        api_key=SecretStr(api_key),
    )

    # Get robot_brain binary path
    rb_path = robot_brain_path or get_robot_brain_path()

    # Configure MCP connection to RoBoT Brain
    mcp_config = {
        "mcpServers": {
            "robot_brain": {
                "command": rb_path,
            }
        }
    }

    # Built-in tools
    tools = [
        Tool(name=TerminalTool.name),
        Tool(name=FileEditorTool.name),
    ]

    # Create agent with optional tool filtering
    agent_kwargs = {
        "llm": llm,
        "tools": tools,
        "mcp_config": mcp_config,
    }

    # Add tool filter if specified
    if filter_tools_regex:
        agent_kwargs["filter_tools_regex"] = filter_tools_regex

    return Agent(**agent_kwargs)


def run_conversation(
    agent: Agent,
    initial_message: str,
    workspace: Optional[str] = None,
    verbose: bool = False,
) -> Conversation:
    """
    Run a conversation with the agent.

    Args:
        agent: The configured agent
        initial_message: First message to send
        workspace: Working directory (defaults to current directory)
        verbose: Print LLM messages

    Returns:
        Completed conversation
    """
    llm_messages = []

    def callback(event: Event):
        if verbose and isinstance(event, LLMConvertibleEvent):
            llm_messages.append(event.to_llm_message())
            logger.info(f"LLM: {str(event)[:200]}...")

    cwd = workspace or os.getcwd()
    conversation = Conversation(
        agent=agent,
        callbacks=[callback] if verbose else [],
        workspace=cwd,
    )

    conversation.set_security_analyzer(LLMSecurityAnalyzer())

    logger.info("Starting conversation with RoBoT Brain agent...")
    conversation.send_message(initial_message)
    conversation.run()

    if verbose:
        print("\n" + "=" * 80)
        print("LLM Messages:")
        for i, msg in enumerate(llm_messages):
            print(f"  [{i}] {str(msg)[:150]}...")

    return conversation


def main():
    """Example usage."""
    import argparse

    parser = argparse.ArgumentParser(description="AI Agent with RoBoT Brain MCP")
    parser.add_argument(
        "--message",
        "-m",
        default="Search memory for any information about Rust programming and list the results.",
        help="Initial message to send to the agent",
    )
    parser.add_argument(
        "--workspace",
        "-w",
        default=None,
        help="Working directory (defaults to current directory)",
    )
    parser.add_argument(
        "--verbose", "-v", action="store_true", help="Print LLM messages"
    )
    parser.add_argument("--model", default=None, help="LLM model to use")
    parser.add_argument(
        "--robot-brain-path", default=None, help="Path to robot_brain binary"
    )

    args = parser.parse_args()

    # Create agent
    logger.info("Initializing RoBoT Brain agent...")
    agent = create_robot_brain_agent(
        llm_model=args.model,
        robot_brain_path=args.robot_brain_path,
    )

    # Run conversation
    conversation = run_conversation(
        agent=agent,
        initial_message=args.message,
        workspace=args.workspace,
        verbose=args.verbose,
    )

    # Report cost
    cost = agent.llm.metrics.accumulated_cost
    logger.info(f"Conversation finished. Total cost: {cost}")


if __name__ == "__main__":
    main()
