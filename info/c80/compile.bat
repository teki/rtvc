@echo off
setlocal EnableExtensions
rem Compile every C80 tutorial example beside this script.
rem Prefers rtvc-c80 / rtvc-tocas on PATH, or RTVC_C80 / RTVC_TOCAS,
rem otherwise cargo run from the repo root.

cd /d "%~dp0"
set "HERE=%cd%"
for %%I in ("%HERE%\..\..") do set "ROOT=%%~fI"
if not exist "%HERE%\out" mkdir "%HERE%\out"

echo Compiling C80 tutorial examples into %HERE%\out

for %%F in ("%HERE%\*.c80") do (
    echo   %%~nF
    call :c80 build "%%F" --origin 0x8000 --emit-asm "%HERE%\out\%%~nF.asm" --emit-bin "%HERE%\out\%%~nF.bin"
    if errorlevel 1 exit /b 1
)

echo   optimize (baseline, --no-optimize)
call :c80 build "%HERE%\optimize.c80" --origin 0x8000 --no-optimize --emit-asm "%HERE%\out\optimize.unopt.asm"
if errorlevel 1 exit /b 1

echo   project/rtvc-c80.toml
call :c80 build "%HERE%\project\rtvc-c80.toml" --emit-asm "%HERE%\out\project.asm" --emit-segments "%HERE%\out\project.toml"
if errorlevel 1 exit /b 1

echo   mixed/rtvc-c80.toml
call :c80 build "%HERE%\mixed\rtvc-c80.toml" --emit-asm "%HERE%\out\mixed.asm" --emit-segments "%HERE%\out\mixed.toml"
if errorlevel 1 exit /b 1
call :tocas "%HERE%\out\mixed.toml"
if errorlevel 1 exit /b 1

echo   tvc-usr/rtvc-c80.toml
call :c80 build "%HERE%\tvc-usr\rtvc-c80.toml" --emit-asm "%HERE%\out\tvc-usr.asm" --emit-segments "%HERE%\out\tvc-usr.toml"
if errorlevel 1 exit /b 1
call :tocas "%HERE%\out\tvc-usr.toml"
if errorlevel 1 exit /b 1

echo   pong/rtvc-c80.toml
call :c80 build "%HERE%\pong\rtvc-c80.toml" --emit-asm "%HERE%\out\pong.asm" --emit-segments "%HERE%\out\pong.toml"
if errorlevel 1 exit /b 1
call :tocas "%HERE%\out\pong.toml"
if errorlevel 1 exit /b 1

echo Done.
exit /b 0

:c80
if defined RTVC_C80 (
    "%RTVC_C80%" %*
    exit /b %ERRORLEVEL%
)
where rtvc-c80 >nul 2>nul
if not errorlevel 1 (
    rtvc-c80 %*
    exit /b %ERRORLEVEL%
)
cargo run --quiet --manifest-path "%ROOT%\Cargo.toml" -p rtvc-c80 -- %*
exit /b %ERRORLEVEL%

:tocas
if defined RTVC_TOCAS (
    "%RTVC_TOCAS%" %*
    exit /b %ERRORLEVEL%
)
where rtvc-tocas >nul 2>nul
if not errorlevel 1 (
    rtvc-tocas %*
    exit /b %ERRORLEVEL%
)
cargo run --quiet --manifest-path "%ROOT%\Cargo.toml" --no-default-features --features asm-toml --bin rtvc-tocas -- %*
exit /b %ERRORLEVEL%
