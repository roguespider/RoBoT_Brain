# RoBoT Brain — Makefile
#
# The wall: `make gate` runs test_suite, which auto-builds robot_brain,
# connects via MCP, runs all tests + code analysis, and enforces the
# AGENTS.md quality wall (0 warnings, 0 code-issues, 0 untested tools).
# Installed as a pre-commit hook (.agents/githooks/pre-commit) so commits
# cannot bypass it.
#
#   make gate        run the full gate (test_suite)
#   make build       build robot_brain only
#   make suite       build + run test_suite
#   make hooks       configure git to use .agents/githooks/ (run once per clone)
#
# Quality is a HARD wall (AGENTS.md: 0 warnings, 0 code-issues, 0 untested
# tools). No ratchet. Fix violations by wiring dead-code pub APIs into real
# callers; never #[allow] or `_`.

.PHONY: gate build suite hooks session

gate: ## Run the full verify gate (the brick wall)
	    @./.agents/scripts/gate.sh

build: ## Build robot_brain (release)
	@. "$$HOME/.cargo/env" 2>/dev/null || true; \
	cargo build --release -p robot_brain

suite: ## Build + run test_suite
	@cd test_suite && cargo build --release && ./target/release/test_suite

hooks: ## Configure git to use .agents/githooks/ (run once per clone)
	@git config core.hooksPath .agents/githooks
	@echo "git hooks installed: core.hooksPath = .agents/githooks"
	@echo "All commits now require 'make gate' to pass (use --no-verify only for doc-only edits)."

session: ## THE NEWSPAPER — run at every session start (reads docs, runs gate, connects MCP, picks first task)
	@./.agents/scripts/session_start.sh
