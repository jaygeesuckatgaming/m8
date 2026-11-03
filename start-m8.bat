@echo off
TITLE m8 - Gem Brain
cd /d "%~dp0"
m8\target\release\m8.exe m8.toml
cmd /k
