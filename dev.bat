@echo off
REM Path of AI — dev launcher
REM Uses MSVC toolchain (x86_64-pc-windows-msvc) with VS 2022 Build Tools.
REM Just runs `tauri dev` — all paths are configured in .cargo/config.toml.

cd /d "%~dp0"
node node_modules\@tauri-apps\cli\tauri.js dev
