@echo off
TITLE m8
cd /d "%~dp0"
m8\target\release\m8.exe m8.toml
cmd /k
