@echo off
REM AeroCore Build Helper — sets up MSVC environment and runs cargo
REM Usage: build.bat [cargo args]

for /f "usebackq tokens=*" %%i in (`"%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath 2^>nul`) do set "VS_PATH=%%i"

if not defined VS_PATH (
    for /f "usebackq tokens=*" %%i in (`dir /b /s "C:\Program Files\Microsoft Visual Studio\*\*\VC\Auxiliary\Build" 2^>nul`) do set "VS_AUX=%%i"
    if defined VS_AUX (
        call "%VS_AUX%\vcvarsall.bat" x64 >nul 2>&1
    ) else (
        echo ERROR: Visual Studio not found. Install VS Build Tools.
        exit /b 1
    )
) else (
    call "%VS_PATH%\VC\Auxiliary\Build\vcvarsall.bat" x64 >nul 2>&1
)

set PATH=%USERPROFILE%\.cargo\bin;%PATH%
cargo %*
