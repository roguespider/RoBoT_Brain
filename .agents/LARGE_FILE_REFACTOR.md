# Large File Refactoring (periodic maintenance)

> Moved here from AGENTS.md on 2026-08-11. This is a periodic maintenance task,
> not a session-start rule. Check it from time to time; do it when a file
> actually needs splitting.

## When to apply

Split a large `.rs` file into a directory module when it mixes multiple
responsibilities AND has grown large. The rough threshold is **~1000 lines**
for genuinely mixed-responsibility files. (A lower ~320-line bar was used
aggressively in the past; many cohesive single-purpose modules above that size
do not need splitting — use judgment, don't split for the sake of it.)

## How to find candidates

```bash
# Files over 1000 lines that may be mixing responsibilities
find src -name "*.rs" -exec wc -l {} + | sort -rn | awk '$1 > 1000'
```

## Refactoring pattern

When splitting a large `.rs` file into modules:

1. Create a directory with the same name as the file (e.g., `engine.rs` ->
   `engine/`)
2. Move original file to `mod.rs` inside the new directory
3. Extract logical groups to separate files (config, types, helpers, etc.)
4. Register modules in `mod.rs` with `pub mod module_name;`
5. Re-export public types for backward compatibility

## Import path migration when splitting files into directories

When a file is split into a directory (e.g., `safety_gate.rs` -> `safety_gate/`),
submodules that previously referenced a sibling module via `super::sibling_module`
must change to `crate::path::to::sibling_module` because the submodule depth
increases by one level. For example, `super::decision` in `safety_gate.rs`
becomes `crate::agent::decision` in `safety_gate/hallucination.rs`. Always
run `cargo build` after splitting to catch these.

## Status (2026-08-10)

No single-file modules over 320 lines remain that mix multiple
responsibilities. The 320-line threshold is aggressive for Rust; many files
above this size are cohesive single-purpose modules that don't need splitting.

## Verify after a refactor

```bash
cargo build --release -p robot_brain          # 0 warnings
python3 test_suite     # 54/54
cd test_suite && cargo build --release && ./target/release/test_suite  # 333/333
```
