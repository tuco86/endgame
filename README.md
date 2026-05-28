<p align="center">
  <img src="assets/endgame-icon.svg" alt="Endgame logo" width="180">
</p>

# Endgame

A tiny Windows tray utility that lets you instantly terminate a stuck or runaway fullscreen game / app with a single global hotkey: **Ctrl+Alt+End**.

> Hit the hotkey → Endgame picks the most likely "game" process → Terminates it safely (never touching blacklisted system processes).

## Why?

Sometimes games freeze, steal focus, or block input (especially in exclusive / borderless fullscreen). Alt+F4 or Task Manager may not work reliably or fast enough. Endgame gives you a deterministic emergency exit.

## Features

- Global hotkey: `Ctrl+Alt+End` (works from anywhere)
- Smart target selection algorithm:
  1. Whitelist processes (immediate kill priority)
  2. Foreground window process
  3. Largest true fullscreen window (monitor-sized)
- Safety blacklist (never killed)
- Configurable via YAML in `%LOCALAPPDATA%/Endgame/config.yaml`
- System tray icon (right-click menu: open config, open log folder, exit)
- Daily rolling log files in `%LOCALAPPDATA%/Endgame/` (last 10 retained, release builds only)
- Attempts to enable `SeDebugPrivilege` for elevated targets

## How It Chooses a Process

1. If any process name matches an entry in `whitelist`, kill the first found.
2. Otherwise take the foreground window's process (if not blacklisted).
3. Otherwise enumerate all top-level windows and choose the one whose bounds exactly match its monitor (largest fullscreen candidate).
4. Skip anything blacklisted.

Blacklisting always wins — if something is in both lists (not recommended), it will not be killed.

## Configuration

Config file location (if created):

```text
%LOCALAPPDATA%\Endgame\config.yaml
```

Behavior:

- An embedded default configuration is always available inside the binary.
- The physical `config.yaml` file is **not** written automatically anymore.
- The file is created on-demand the first time you choose tray menu → "Configurations" (or if you create it manually).
- If the file does not exist at startup, Endgame uses the embedded defaults in memory.

Example file (what gets written when first created):

```yaml
blacklist: ["explorer.exe", "dwm.exe", "csrss.exe", "winlogon.exe"]
whitelist: ["game.exe", "someapp.exe"]
```

Fields:

- `blacklist` — Processes never terminated (safety first).
- `whitelist` — High-priority targets. If any are running, the first match is terminated immediately before other heuristics.

Editing Tips:

- Open via tray → "Configurations" to auto-create and edit.
- Remove entries you do not want; names are case-insensitive comparisons on the executable filename.
- Changes take effect the next time you trigger the hotkey (no restart required unless you add/remove the file itself).

## Install / Build

### Prerequisites

- Rust toolchain (stable) on Windows (MSVC)

### Build

```powershell
# Clone and build
git clone https://github.com/tuco86/endgame.git
cd endgame
cargo build --release

# Binary will be at:
# .\target\release\endgame.exe
```

Run it (release build detaches from console):

```powershell
./target/release/endgame.exe
```

The Endgame icon (broken Xbox controller pierced by a sword) appears in the system tray. Use the hotkey; watch the terminal (debug) or check the log folder (release).

### Auto-Start (Optional)

Create a shortcut to the release binary and place it in:

```text
%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
```

## Usage

1. Launch Endgame.
2. Start / switch to a game.
3. If it freezes or misbehaves, press `Ctrl+Alt+End`.
4. The target process is terminated according to the selection rules.
5. Adjust `whitelist` / `blacklist` as needed for your library.

## Security / Safety Notes

- Uses `TerminateProcess` — this is a hard kill (no graceful shutdown)
- Never touches blacklisted system-critical processes (defaults provided)
- Enabling `SeDebugPrivilege` may require running as Administrator to affect elevated processes

## Roadmap / Future Ideas

- Soft kill attempt (send `WM_CLOSE`) before force terminate
- Process tree kill (children of the target)
- Configurable hotkey
- Better heuristics (GPU usage, focus history)
- Notification balloons / richer status feedback

## Architecture Overview

- **Tray layer:** `tray.rs` (winit + tray-icon) handles menu, tooltip, and user-event dispatch
- **Hotkey thread:** `hotkey.rs` registers and listens via a Win32 message loop on a worker thread; presses are forwarded to the winit event loop via `EventLoopProxy`
- **Process logic:** `process.rs` handles privilege escalation, window enumeration, selection & termination
- **Config loader:** `config.rs` embeds default YAML and persists user overrides
- **Logging:** `log.rs` uses `simplelog` (terminal, debug) or `tracing-appender` (daily rolling files, release)
- **Main:** wires modules, enables debug privilege, starts event loop

## Limitations

- Does not currently attempt graceful shutdown
- Only one global hotkey, not yet user-configurable
- Multi-monitor fullscreen heuristic picks the largest monitor-sized window (improvable)

## Contributing

Issues and PRs welcome. Keep changes focused and minimal. Before submitting:

- Run `cargo fmt` & `cargo clippy` (if configured)
- Test with at least one fullscreen game and a windowed app

## License

MIT License — see [LICENSE](./LICENSE).

## Attribution / AI Assistance

Most of this project (initial scaffolding, Win32 integration, refactors, and documentation) was generated with assistance from an AI coding tool (Copilot / GPT-5 in VS Code) and then manually reviewed / edited. This note is provided purely for transparency; there are no contributor disclosure expectations.

---

"End the game, not your session." Enjoy.
