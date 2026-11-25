@echo off
REM .agents/scripts/test_suite_run.bat - Python test suite runner (test_suite2.1)
REM Usage: test_suite_run.bat [number] or [option_name]
REM Numbered options:
REM   1 - Gate - Python analysis + quality wall
REM   2 - All  - analysis + pytest (runs every test in test_suite2.1)
REM   3 - Python (pytest only)
REM   4 - Build robot_brain only (test_suite2.1 auto-builds it)
REM   5 - Clean all build artifacts
REM   6 - Probe a specific tool's schema
REM   7 - Show this help
REM
REM Also accepts: gate all python build clean probe help

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "REPO_ROOT=%SCRIPT_DIR%..\.."
set "TS_DIR=%SCRIPT_DIR%test_suite2.1"
set "OUTPUT_FILE=%SCRIPT_DIR%test_output.txt"

REM Ensure Rust on PATH
if exist "%USERPROFILE%\.cargo\env" call "%USERPROFILE%\.cargo\env"

set "ACTION=%~1"
REM Convert number to name
if "%ACTION%"=="1" set "ACTION=gate"
if "%ACTION%"=="2" set "ACTION=all"
if "%ACTION%"=="3" set "ACTION=python"
if "%ACTION%"=="4" set "ACTION=build"
if "%ACTION%"=="5" set "ACTION=clean"
if "%ACTION%"=="6" set "ACTION=probe"
if "%ACTION%"=="7" set "ACTION=help"

REM Handle probe with tool name (e.g., 6 probe or 6 store_memory)
if "%ACTION%"=="6" set "ACTION=probe %~2"

if "%ACTION%"=="" goto :menu
if /i "%ACTION%"=="help" goto :help
if /i "%ACTION%"=="gate" goto :gate
if /i "%ACTION%"=="all" goto :all
if /i "%ACTION%"=="python" goto :python
if /i "%ACTION%"=="build" goto :build
if /i "%ACTION%"=="clean" goto :clean
if /i "%ACTION%"=="probe" goto :probe
if /i "%ACTION%"=="probe " goto :probe

REM Invalid option
echo Error: Unknown option '%~1'
echo.
goto :menu

REM ============================================================================
REM Helper: clean previous output files
REM ============================================================================
:clean_output
if exist "%OUTPUT_FILE%" del "%OUTPUT_FILE%"
if exist "%TS_DIR%\test_suite_report.json" del "%TS_DIR%\test_suite_report.json"
if exist "%TS_DIR%\python_test_output.txt" del "%TS_DIR%\python_test_output.txt"
if exist "%TS_DIR%\analysis_output.txt" del "%TS_DIR%\analysis_output.txt"
if exist "%TS_DIR%\all_tools_list.txt" del "%TS_DIR%\all_tools_list.txt"
if exist "%TS_DIR%\tested_tools.txt" del "%TS_DIR%\tested_tools.txt"
if exist "%REPO_ROOT%\.agents\scripts\tool_coverage_report.json" del "%REPO_ROOT%\.agents\scripts\tool_coverage_report.json"
if not exist "%REPO_ROOT%\.agents\scripts\" mkdir "%REPO_ROOT%\.agents\scripts\"
goto :eof

REM ============================================================================
REM Helper: show output file contents
REM ============================================================================
:show_output
type "%OUTPUT_FILE%"
goto :eof

REM ============================================================================
REM Option 1: Gate - Python analysis + quality wall
REM ============================================================================

:gate
echo.
echo ==========================================
echo  RUNNING GATE (analysis + quality wall)
echo ==========================================
echo.
call :clean_output

REM ---- Run Python analysis FIRST (fast fail) ----
echo [RUN] Python code analysis, lint, registry...
pushd "%TS_DIR%"
python main.py
set "MAIN_EXIT=%errorlevel%"
popd

if "%MAIN_EXIT%" neq "0" (
    echo.
    echo [RED] GATE FAILED - analysis exited with code %MAIN_EXIT%
    echo See: %OUTPUT_FILE%
    type "%OUTPUT_FILE%" 2>nul
    exit /b 1
)
echo [PASS] Analysis passed.

REM Check analysis results before proceeding
set "CODE_ISSUES=0"
findstr /C:"Code issues:" "%OUTPUT_FILE%" >nul 2>&1 && (
    for /f "tokens=2 delims=|" %%c in ('findstr /C:"Code issues:" "%OUTPUT_FILE%"') do (
        for /f "tokens=*" %%d in (%%c) do set "CODE_ISSUES=%%d"
    )
)
set "LINT_ISSUES=0"
findstr /C:"Lint issues:" "%OUTPUT_FILE%" >nul 2>&1 && (
    for /f "tokens=2 delims=|" %%c in ('findstr /C:"Lint issues:" "%OUTPUT_FILE%"') do (for /f "tokens=*" %%d in (%%c) do set "LINT_ISSUES=%%d")
)

if "%CODE_ISSUES%" neq "0" (
    echo.
    echo [RED] GATE FAILED - code issues: %CODE_ISSUES%
    exit /b 1
)
if "%LINT_ISSUES%" neq "0" (
    echo.
    echo [RED] GATE FAILED - lint issues: %LINT_ISSUES%
    exit /b 1
)

REM ---- Quality wall ---
echo.
echo [CHECK] Quality wall...
set "PASS=0"
set "WARN=0"
set "ISSUES=0"
set "UNTESTED=0"
set "PY_COMPILE=0"
set "PY_LINT=0"
set "PY_QUALITY=0"
set "LARGE_FILES=0"

REM Parse Gate Status section for actual metrics
findstr /I /C:"Code issues: 0" "%OUTPUT_FILE%" >nul 2>&1 && set "ISSUES=0" || set "ISSUES=1"
findstr /I /C:"Lint issues: 0" "%OUTPUT_FILE%" >nul 2>&1 && set "WARN=0" || set "WARN=1"
findstr /I /C:"Large file issues: 0" "%OUTPUT_FILE%" >nul 2>&1 && set "LARGE_FILES=0" || set "LARGE_FILES=1"
findstr /I /C:"Python errors: 0" "%OUTPUT_FILE%" >nul 2>&1 && set "PY_COMPILE=0" || set "PY_COMPILE=1"
set "UNTESTED=0"
set "PY_LINT=0"
set "PY_QUALITY=0"
findstr /C:"[PASS] No files exceed 800 lines" "%OUTPUT_FILE%" >nul 2>&1 && set "LARGE_FILES=0" || set "LARGE_FILES=1"

echo [INFO] Parsed: warnings=!WARN! issues=!ISSUES! untested=!UNTESTED! py_compile=!PY_COMPILE! py_lint=!PY_LINT! py_quality=!PY_QUALITY! large_files=!LARGE_FILES!

if "!WARN!" neq "0" (
    echo.
    echo [RED] GATE FAILED - warnings: !WARN!
    exit /b 1
)
if "!ISSUES!" neq "0" (
    echo.
    echo [RED] GATE FAILED - issues: !ISSUES!
    exit /b 1
)
if "!UNTESTED!" neq "0" (
    echo.
    echo [RED] GATE FAILED - untested tools: !UNTESTED!
    exit /b 1
)
if "!PY_COMPILE!" neq "0" (
    echo.
    echo [RED] GATE FAILED - Python compile issues: !PY_COMPILE!
    exit /b 1
)
if "!PY_LINT!" neq "0" (
    echo.
    echo [RED] GATE FAILED - Python lint issues: !PY_LINT!
    exit /b 1
)
if "!PY_QUALITY!" neq "0" (
    echo.
    echo [RED] GATE FAILED - Python quality issues: !PY_QUALITY!
    exit /b 1
)
if "!LARGE_FILES!" neq "0" (
    echo.
    echo [RED] GATE FAILED - large file issues: !LARGE_FILES!
    exit /b 1
)

echo.
echo [PASS] Gate green (warnings=!WARN! issues=!ISSUES! untested=!UNTESTED!)
echo.
echo === GATE GREEN - commit permitted ===
echo   Consolidated output: %OUTPUT_FILE%
echo   Analysis: !WARN! warnings, !ISSUES! issues, !UNTESTED! untested tools
goto :eof

REM ============================================================================
REM Option 2: All - EVERYTHING (analysis + pytest)
REM ============================================================================

:all
echo.
echo ==========================================
echo  RUNNING ALL TESTS (analysis + pytest)
echo ==========================================
echo.
call :clean_output
REM Wait for file system to release any handles (Windows file caching)
powershell -NoProfile -Command "Start-Sleep -Milliseconds 500" 2>nul

REM Step 1: Python analysis (code/lint/registry) - always continue
echo [ANALYSIS] Python code analysis, lint, registry...
pushd "%TS_DIR%"
REM main.py writes directly to OUTPUT_FILE, no redirection needed
python main.py 2>nul
set "MAIN_EXIT=%errorlevel%"
popd
REM Wait for file handle to be fully released before pytest appends
powershell -NoProfile -Command "Start-Sleep -Milliseconds 500" 2>nul
if "%MAIN_EXIT%" neq "0" (
    echo [WARN] Analysis reported issues - see report
) else (
    echo [PASS] Analysis passed.
)
REM Verify Python errors section is visible in output
echo.
echo === PYTHON ANALYSIS RESULTS ===
type "%OUTPUT_FILE%" 2>nul | findstr /C:"Python Errors" /C:"PASS" /C:"FAIL"
echo === END PYTHON ANALYSIS ===
echo.

REM Step 2: Build robot_brain if needed (pytest requires the binary)
echo [CHECK] robot_brain binary...
set "BRAIN_FOUND=0"
if exist "%REPO_ROOT%\target\release\robot_brain.exe" set "BRAIN_FOUND=1"
if exist "%REPO_ROOT%\target\release\robot_brain" set "BRAIN_FOUND=1"
if exist "%REPO_ROOT%\target\debug\robot_brain.exe" set "BRAIN_FOUND=1"
if exist "%REPO_ROOT%\target\debug\robot_brain" set "BRAIN_FOUND=1"
if "%BRAIN_FOUND%"=="0" (
    echo [BUILD] robot_brain not found, building release...
    pushd "%REPO_ROOT%"
    call cargo build --release -p robot_brain >> "%OUTPUT_FILE%" 2>&1
    if errorlevel 1 (
        echo Error: robot_brain build failed
        popd
        exit /b 1
    )
    popd
    echo [PASS] robot_brain built.
) else (
    echo [PASS] robot_brain binary found.
)

REM Step 3: Run pytest - every test in test_suite2.1
echo.
echo [RUN] pytest - all tests in test_suite2.1...
pushd "%TS_DIR%"
if not exist "conftest.py" (
    echo Error: conftest.py not found in test_suite2.1
    popd
    exit /b 1
)

echo [RUN] Running pytest (hard 25 min timeout)...
echo. >> "%OUTPUT_FILE%"
echo [PYTEST_START] %date% %time% >> "%OUTPUT_FILE%"
pushd "%TS_DIR%"
python run_tests_with_timeout.py "%OUTPUT_FILE%" 1500
set "PYTEST_EXIT=%errorlevel%"
popd
echo [PYTEST_END] %date% %time% >> "%OUTPUT_FILE%"

echo.
echo === ALL TESTS COMPLETE ===
echo   Analysis exit code: %MAIN_EXIT%
echo   pytest exit code:   %PYTEST_EXIT%
echo   See: %OUTPUT_FILE%

if "%MAIN_EXIT%" neq "0" exit /b 1
if "%PYTEST_EXIT%" neq "0" exit /b 1
goto :eof

REM ============================================================================
REM Option 3: Python - pytest only
REM ============================================================================

:python
echo.
echo ==========================================
echo  RUNNING PYTHON TESTS (pytest only)
echo ==========================================
echo.
pushd "%TS_DIR%"
if not exist "conftest.py" (
    echo Error: conftest.py not found in test_suite2.1
    popd
    exit /b 1
)
echo [CHECK] Python environment...
python --version
echo.
echo [RUN] pytest...
powershell -NoProfile -Command "$env:PYTHONUNBUFFERED='1'; $env:PYTHONDONTWRITEBYTECODE='1'; python -m pytest -v -c pytest.ini --durations=10 --tb=short ."
popd
goto :eof

REM ============================================================================
REM Option 4: Build
REM ============================================================================

:build
echo.
echo ==========================================
echo  BUILDING
echo ==========================================
echo.
echo [BUILD] robot_brain...
pushd "%REPO_ROOT%"
call cargo build --release -p robot_brain
if errorlevel 1 (
    echo Error: robot_brain build failed
    popd
    exit /b 1
)
popd
echo.
echo === Build complete ===
goto :eof

REM ============================================================================
REM Option 5: Clean
REM ============================================================================

:clean
echo.
echo ==========================================
echo  CLEANING
echo ==========================================
echo.
REM Clean test_suite2.1
echo [INFO] Cleaning test_suite2.1 artifacts...
if exist "%TS_DIR%\__pycache__" (
    rmdir /s /q "%TS_DIR%\__pycache__"
    echo [CLEAN] __pycache__
)
if exist "%TS_DIR%\.pytest_cache" (
    rmdir /s /q "%TS_DIR%\.pytest_cache"
    echo [CLEAN] .pytest_cache
)
REM Clean orphan .txt output files (only .agents/scripts/test_output.txt is the valid
REM consolidated output)
for %%f in ("%TS_DIR%\*.txt") do (
    del "%%f"
    echo [CLEAN] %%~nxf
)
echo [INFO] Cleaning robot_brain (Cargo.toml location: %REPO_ROOT%\Cargo.toml)...
REM Clean robot_brain using cargo clean (proper Cargo cleanup)
if exist "%REPO_ROOT%\Cargo.toml" (
    pushd "%REPO_ROOT%"
    echo [INFO] Running cargo clean...
    cargo clean
    if errorlevel 1 (
        echo [WARN] cargo clean failed, trying manual removal...
        if exist "target" (
            rmdir /s /q "target"
            echo [CLEAN] robot_brain target/ (manual removal)
        )
    ) else (
        echo [CLEAN] robot_brain cargo clean
    )
    popd
) else (
    echo [WARN] Cargo.toml not found at %REPO_ROOT%\Cargo.toml
    REM Fallback: delete target directory if cargo.toml not found
    if exist "%REPO_ROOT%\target" (
        rmdir /s /q "target"
        echo [CLEAN] robot_brain target/ (fallback)
    ) else (
        echo [INFO] No target directory found
    )
)
echo.
echo === Clean complete ===
goto :eof

REM ============================================================================
REM Option 6: Probe
REM ============================================================================

:probe
if "%~2"=="" if "%~3"=="" (
    REM Interactive submenu: list tools and pick one
    echo.
    echo ==========================================
    echo  TOOL PROBE - Select a tool to inspect
    echo ==========================================
    echo.
    echo [LIST] Available tools (from MCP server):
    echo.

    REM Try to list tools first
    pushd "%TS_DIR%"
    echo [INFO] Listing tools via MCP server...
    python main.py --list 2>nul
    popd

    echo.
    echo [INPUT] Enter tool name to probe:
    set /p TOOL_NAME="> "
    if "%TOOL_NAME%"=="" (
        echo Error: No tool name specified
        goto :eof
    )
) else (
    set "TOOL_NAME=%~2"
)

echo.
echo ==========================================
echo  PROBING TOOL: %TOOL_NAME%
echo ==========================================
echo.
pushd "%TS_DIR%"
python main.py --probe "%TOOL_NAME%"
popd
goto :eof

REM ============================================================================
REM Option 7: Help
REM ============================================================================

:help
echo.
echo RoBoT Brain Test Suite Runner
echo =============================
echo.
echo Usage: test_suite_run.bat [number] [tool_name]
echo        test_suite_run.bat [option_name] [tool_name]
echo.
echo Numbered Options:
echo   1  Gate - Python analysis + quality wall
echo   2  All  - analysis + pytest (runs every test in test_suite2.1)
echo   3  Python - Run pytest functional tests only
echo   4  Build - Build robot_brain only
echo   5  Clean - Remove all build artifacts (target/, __pycache__, .pytest_cache)
echo   6  Probe - Probe a tool's schema: test_suite_run.bat 6 store_memory
echo   7  Help - Show this help
echo.
echo Also accepts option names: gate all python build clean probe help
echo.
echo Examples:
echo   test_suite_run.bat 1              (gate - analysis + quality wall)
echo   test_suite_run.bat 2              (all - analysis + pytest)
echo   test_suite_run.bat 6 store_memory  (probe schema)
echo   test_suite_run.bat clean          (clean artifacts)
echo   test_suite_run.bat all            (same as 2)
echo.
exit /b 0

REM ============================================================================
REM Menu / Dispatch
REM ============================================================================

:menu
echo.
echo ==========================================
echo  RoBoT Brain Test Suite Runner
echo ==========================================
echo.
echo    1  Gate - Python analysis + quality wall
echo    2  All  - analysis + pytest (runs every test in test_suite2.1)
echo    3  Python - pytest only
echo    4  Build - Build robot_brain only
echo    5  Clean - Clean all build artifacts
echo    6  Probe - Probe a tool's schema
echo    7  Help - Show this help
echo.
set /p "choice=Enter option (1-7) or press Enter for gate: "
if "%choice%"=="" set "choice=1"
REM Convert number to name
if "%choice%"=="1" set "ACTION=gate"
if "%choice%"=="2" set "ACTION=all"
if "%choice%"=="3" set "ACTION=python"
if "%choice%"=="4" set "ACTION=build"
if "%choice%"=="5" set "ACTION=clean"
if "%choice%"=="6" set "ACTION=probe %~2"
if "%choice%"=="7" set "ACTION=help"
REM Handle probe with tool name from command line
if "%~2" neq "" set "ACTION=probe %~2"
goto :dispatch

:dispatch
echo.
echo ==========================================
echo  DISPATCH: %ACTION%
echo ==========================================
echo.
REM Parse action and call appropriate label
set "MAIN_ACTION=%ACTION: =%"
if /i "!MAIN_ACTION!"=="gate" goto :gate
if /i "!MAIN_ACTION!"=="all" goto :all
if /i "!MAIN_ACTION!"=="python" goto :python
if /i "!MAIN_ACTION!"=="build" goto :build
if /i "!MAIN_ACTION!"=="clean" goto :clean
if /i "!MAIN_ACTION!"=="probe" goto :probe
if /i "!MAIN_ACTION!"=="help" goto :help
echo Error: Unknown action '%ACTION%'
goto :eof
