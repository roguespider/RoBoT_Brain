@echo off
REM .agents/scripts/test_suite_run.bat — Python test suite runner (test_suite2.1)
REM Usage: test_suite_run.bat [number] or [option_name]
REM Numbered options:
REM   1 - Full gate (Python analysis + pytest + report)
REM   3 - Python pytest suite
REM   4 - Build robot_brain only (test_suite2.1 auto-builds it)
REM   5 - Clean all build artifacts
REM   6 - Probe a specific tool's schema
REM   7 - Show this help
REM
REM Also accepts: all python build clean probe help

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "REPO_ROOT=%SCRIPT_DIR%..\..\.."
set "TS_DIR=%SCRIPT_DIR%test_suite2.1"

REM Ensure Rust on PATH
if exist "%USERPROFILE%\.cargo\env" call "%USERPROFILE%\.cargo\env"

set "ACTION=%~1"
REM Convert number to name
if "%ACTION%"=="1" set "ACTION=all"
if "%ACTION%"=="3" set "ACTION=python"
if "%ACTION%"=="4" set "ACTION=build"
if "%ACTION%"=="5" set "ACTION=clean"
if "%ACTION%"=="6" set "ACTION=probe"
if "%ACTION%"=="7" set "ACTION=help"

REM Handle probe with tool name (e.g., 6 probe or 6 store_memory)
if "%ACTION%"=="6" set "ACTION=probe %~2"

if "%ACTION%"=="" goto :menu
if /i "%ACTION%"=="help" goto :help
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

:menu
echo.
echo ==========================================
echo  RoBoT Brain Test Suite Runner
echo ==========================================
echo.
echo    1  Full gate (Python analysis + pytest + report)
echo    3  Python pytest suite
echo    4  Build robot_brain only
echo    5  Clean all build artifacts
echo    6  Probe a tool's schema
echo    7  Show this help
echo.
set /p "choice=Enter option (1-7) or press Enter for full gate: "
if "%choice%"=="" set "choice=1"
REM Convert number to name
if "%choice%"=="1" set "ACTION=all"
if "%choice%"=="3" set "ACTION=python"
if "%choice%"=="4" set "ACTION=build"
if "%choice%"=="5" set "ACTION=clean"
if "%choice%"=="6" set "ACTION=probe %~2"
if "%choice%"=="7" set "ACTION=help"
REM Handle probe with tool name from command line
if "%~2" neq "" set "ACTION=probe %~2"
goto :dispatch

:help
echo.
echo RoBoT Brain Test Suite Runner
echo =============================
echo.
echo Usage: test_suite_run.bat [number] [tool_name]
echo        test_suite_run.bat [option_name] [tool_name]
echo.
echo Numbered Options:
echo   1  Full gate (Python analysis + pytest + report)
echo   3  Run Python pytest functional tests
echo   4  Build robot_brain only
echo   5  Remove all build artifacts (target/, __pycache__, .pytest_cache)
echo   6  Probe a tool's schema: test_suite_run.bat 6 store_memory
echo   7  Show this help
echo.
echo Also accepts option names: all python build clean probe help
echo.
echo Examples:
echo   test_suite_run.bat 1              (full gate)
echo   test_suite_run.bat 6 store_memory  (probe schema)
echo   test_suite_run.bat clean          (clean artifacts)
echo   test_suite_run.bat all            (same as 1)
echo.
exit /b 0

:all
echo.
echo ==========================================
echo  RUNNING FULL GATE
echo ==========================================
echo.
echo [RUN] Python analysis + tests...
call bash "%SCRIPT_DIR%make.sh" gate
goto :eof

:python
echo.
echo ==========================================
echo  RUNNING PYTHON TESTS
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
echo [ANALYSIS] Running code analyzer + lint + registry...
python main.py
echo.
echo [RUN] pytest...
python -m pytest -v -c pytest.ini .
popd
goto :eof

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
echo [INFO] Cleaning robot_brain (Cargo.toml location: %REPO_ROOT%\Cargo.toml)...n
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
        rmdir /s /q "%REPO_ROOT%\target"
        echo [CLEAN] robot_brain target/ (fallback)
    ) else (
        echo [INFO] No target directory found
    )
)
echo.
echo === Clean complete ===
goto :eof

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

:dispatch
echo.
echo ==========================================
echo  DISPATCH: %ACTION%
echo ==========================================
echo.
REM Parse action and call appropriate label
set "MAIN_ACTION=%ACTION: =%"
if /i "!MAIN_ACTION!"=="all" goto :all
if /i "!MAIN_ACTION!"=="python" goto :python
if /i "!MAIN_ACTION!"=="build" goto :build
if /i "!MAIN_ACTION!"=="clean" goto :clean
if /i "!MAIN_ACTION!"=="probe" goto :probe
if /i "!MAIN_ACTION!"=="help" goto :help
echo Error: Unknown action '%ACTION%'
goto :eof
