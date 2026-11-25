#!/usr/bin/env python3
"""
Run pytest with a timeout and capture output to a file.
Runs each test file in a separate subprocess to ensure test isolation.
Usage: run_tests_with_timeout.py <output_file> <timeout_seconds>
"""

import asyncio
import glob
import os
import sys
import time


def _write_to_file(output_file: str, content: str) -> None:
    """Write content to file (blocking, called from sync context)."""
    with open(output_file, "a", encoding="utf-8") as f:
        f.write(content)


def discover_test_files(test_dir: str) -> list:
    """Discover test files in the test directory."""
    patterns = ["test_*.py", "main.py"]
    test_files = []
    for pattern in patterns:
        test_files.extend(glob.glob(os.path.join(test_dir, pattern)))
    test_files = [f for f in test_files if os.path.isfile(f) and f.endswith(".py")]
    return sorted(set(test_files))


async def run_pytest_file(
    test_file: str,
    test_dir: str,
    output_file: str,
    timeout: float,
    global_start: float,
) -> int:
    """Run a single test file with its own pytest process."""
    elapsed = time.time() - global_start
    remaining = timeout - elapsed
    if remaining <= 10:
        _write_to_file(
            output_file,
            f"\n[TIMEOUT] Not enough time for {test_file}\n",
        )
        return -1

    file_timeout = min(remaining - 5, 120)

    try:
        process = await asyncio.create_subprocess_shell(
            f"{sys.executable} -m pytest -v -c pytest.ini "
            f'--durations=10 --tb=short "{test_file}"',
            cwd=test_dir,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )

        _write_to_file(
            output_file,
            f"\n{'=' * 70}\n"
            f"[FILE_START] {os.path.basename(test_file)} "
            f"at {time.ctime()}\n"
            f"{'=' * 70}\n",
        )

        # Read all output at once with timeout
        try:
            output, _ = await asyncio.wait_for(
                process.communicate(), timeout=file_timeout
            )
            if output:
                _write_to_file(output_file, output.decode(errors="replace"))
            return process.returncode or 1
        except TimeoutError:
            process.kill()
            await process.wait()
            fname = os.path.basename(test_file)
            _write_to_file(
                output_file,
                f"\n[FILE_TIMEOUT] {fname} exceeded {file_timeout:.0f}s\n",
            )
            return -1

    except TimeoutError as e:
        _write_to_file(output_file, f"\n[TIMEOUT ERROR] {e}\n")
        return -1
    except OSError as e:
        _write_to_file(output_file, f"\n[OS ERROR] {e}\n")
        return -1
    except ValueError as e:
        _write_to_file(output_file, f"\n[VALUE ERROR] {e}\n")
        return -1


async def run_pytest(timeout: float, output_file: str) -> int:
    """Run pytest with a timeout, writing output to output_file."""
    global_start = time.time()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    test_dir = script_dir

    test_files = discover_test_files(test_dir)
    if not test_files:
        _write_to_file(
            output_file,
            "\n[ERROR] No test files found in " + test_dir + "\n",
        )
        return -1

    _write_to_file(
        output_file,
        f"\n{'=' * 70}\n"
        f"[PYTEST_OUTPUT] Started at {time.ctime()}\n"
        f"Running {len(test_files)} test files in isolation\n"
        f"{'=' * 70}\n\n",
    )

    total = len(test_files)
    passed = 0
    failed = 0

    for i, test_file in enumerate(test_files, 1):
        sys.stderr.write(f"Running test {i}/{total}: {os.path.basename(test_file)}\n")
        sys.stderr.flush()

        result = await run_pytest_file(
            test_file, test_dir, output_file, timeout, global_start
        )

        if result == 0:
            passed += 1
        else:
            elapsed = time.time() - global_start
            if elapsed > timeout:
                _write_to_file(
                    output_file,
                    f"\n[GLOBAL_TIMEOUT] Stopping after {int(timeout)}s\n",
                )
                break
            failed += 1

    _write_to_file(
        output_file,
        f"\n{'=' * 70}\n"
        f"[PYTEST_SUMMARY] Completed at {time.ctime()}\n"
        f"Files run: {total}, Passed: {passed}, Failed: {failed}\n"
        f"{'=' * 70}\n",
    )

    return 0 if failed == 0 else 1


def main():
    if len(sys.argv) < 3:
        sys.stderr.write(f"Usage: {sys.argv[0]} <output_file> <timeout_seconds>\n")
        sys.exit(1)

    output_file = sys.argv[1]
    timeout = int(sys.argv[2])

    os.makedirs(
        os.path.dirname(output_file) if os.path.dirname(output_file) else ".",
        exist_ok=True,
    )

    return asyncio.run(run_pytest(timeout, output_file))


if __name__ == "__main__":
    sys.exit(main())
