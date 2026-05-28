# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Endgame is a Windows-only Rust tray utility that registers a global `Ctrl+Alt+End` hotkey to force-terminate a stuck fullscreen game/app. Edition 2024, targets the MSVC toolchain. Cannot be meaningfully built or run on non-Windows hosts — all process/window logic goes through Win32.

## Commands

```powershell
cargo build                # debug build, attaches a console (logs to terminal)
cargo build --release      # release build, detached (windows_subsystem = "windows")
cargo run                  # debug run with terminal logging
cargo fmt
cargo clippy
```

There are currently no tests in the repo, so `cargo test` is a no-op.

Debug vs release behavior is controlled at compile time, not by flags:
- Debug: console subsystem, logs via `simplelog` to the terminal.
- Release: windows subsystem (no console), logs via `tracing-appender` daily-rotating files in `%LOCALAPPDATA%\Endgame\` (kept: last 10).

## Architecture

`main.rs` wires four modules and then hands control to a `winit` event loop:

1. **`log.rs`** — `init()` is `#[cfg]`-split: terminal logger in debug, rolling file logger in release. The release init intentionally `mem::forget`s the `WorkerGuard` to keep the non-blocking writer alive for the life of the process.
2. **`config.rs`** — `Config { whitelist, blacklist }` (YAML). `src/config.yaml` is embedded via `include_str!` and used as the in-memory default. The on-disk file at `%LOCALAPPDATA%\Endgame\config.yaml` is only written when the user clicks the "Configurations" tray entry (`ensure_config_file`). Loading at startup falls back to the embedded default if the file is missing — do not change `load_config` to auto-write the file.
3. **`hotkey.rs`** — Spawns a dedicated worker thread that calls Win32 `RegisterHotKey` with no owner window and runs a `GetMessageW` pump, forwarding `WM_HOTKEY` events through an `mpsc::Receiver<()>`. Because there is no window, messages target the thread itself; that thread must keep running for the hotkey to stay registered.
4. **`process.rs`** — Target selection in priority order: `find_whitelist` (toolhelp snapshot) → `foreground_process` (`GetForegroundWindow`) → `fullscreen_fallback` (`EnumWindows` for visible non-toolwindow windows whose rect exactly matches their monitor). Blacklist always vetoes. `enable_debug_privilege` is best-effort and may fail without elevation. Termination is `TerminateProcess` (hard kill — no `WM_CLOSE` first).
5. **`tray.rs`** — `TrayApp` implements `winit::ApplicationHandler`. The event loop runs in `ControlFlow::Poll`, and `about_to_wait` drains both the tray menu channel (`MenuEvent::receiver()`) and the hotkey channel each tick. The hotkey-press handler is what actually calls `process::find_target` + `process::terminate`.

Key cross-module invariants when modifying:
- `Config` is wrapped in `Arc<Config>` and read-only after startup; reloading on hotkey requires re-reading from disk explicitly.
- The hotkey listener thread is detached and never joined — its `Receiver` end lives on `TrayApp`.
- `windows_subsystem = "windows"` is gated on `not(debug_assertions)`; keep this attribute at the top of `main.rs` or release builds will spawn a console.
- Two crates are used for Win32 bindings: `windows` (for keyboard/window constants) and `windows-sys` (for everything else). Match the one already used in the file you're editing rather than mixing.

## Conventions from README

- Process name matches are case-insensitive on the executable filename only (no path matching).
- Blacklist entries always win over whitelist entries.
- Default blacklist includes `explorer.exe`, `dwm.exe`, `csrss.exe`, `winlogon.exe` — do not remove these from `src/config.yaml` without a strong reason.
