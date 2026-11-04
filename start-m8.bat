@echo off
TITLE m8 - Gem Brain
cd /d "%~dp0"

rem (m8 checks the voice adapter at startup: wait a moment for it, then say what's missing)
powershell -NoProfile -Command "for ($i=0; $i -lt 15; $i++) { try { $r = Invoke-WebRequest -UseBasicParsing -TimeoutSec 1 http://127.0.0.1:8091/health; if ($r.StatusCode -eq 200) { exit 0 } } catch {} ; Start-Sleep -Milliseconds 700 }; exit 1"
if errorlevel 1 (
    echo.
    echo   The m8 TTS adapter is not running on http://127.0.0.1:8091.
    echo   Start "m8 TTS adapter" in the Gem panel ^(Games^), then run this again.
    echo.
    cmd /k
    exit /b
)

m8\target\release\m8.exe m8.toml
cmd /k
