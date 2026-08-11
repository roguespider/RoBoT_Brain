# RoBoT Brain — Makefile
#
# The wall: `make gate` runs the full verify gate (build -> connect to
# robot_brain -> test_suite) and aborts on the first failure. Installed as
# a pre-commit hook (.githooks/pre-commit) so commits cannot bypass it.
#
#   make gate        run the full gate (build + live + suite)
#   make build       build robot_brain only
#   make live        connect to robot_brain and run live_test_all.py
#   make suite       build + run test_suite
#   make hooks       configure git to use .githooks/ (run once per clone)
#
# Quality is a HARD wall (AGENTS.md: 0 warnings, 0 code-issues, 0 untested
# tools). No ratchet. Fix violations by wiring dead-code pub APIs into real
# callers; never #[allow] or `_`.

.PHONY: gate build live suite hooks

gate: ## Run the full verify gate (the brick wall)
	@./scripts/gate.sh

build: ## Build robot_brain (release)
	@. "$$HOME/.cargo/env" 2>/dev/null || true; \
	cargo build --release -p robot_brain

live: ## Connect to robot_brain and run live_test_all.py (54/54)
	@./scripts/gate.sh 2>/dev/null || true
	@python3 .agents/live_test/live_test_all.py

suite: ## Build + run test_suite
	@cd test_suite && cargo build --release && ./target/release/test_suite

hooks: ## Configure git to use .githooks/ (run once per clone)
	@git config core.hooksPath .githooks
	@echo "git hooks installed: core.hooksPath = .githooks"
	@echo "All commits now require 'make gate' to pass (use --no-verify only for doc-only edits)."
