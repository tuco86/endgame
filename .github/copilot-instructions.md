# Endgame - AI Coding Agent Instructions

## Project Overview
**Endgame** is a minimal Windows tray application that provides an emergency kill hotkey (`Ctrl+Alt+End`) to terminate games or fullscreen applications. Built in Rust using Win32 APIs for maximum reliability and configurable via YAML.

## Architecture

### Core Components
- **Tray Application**: System tray icon with no visible UI
- **Global Hotkey Handler**: Registers and responds to `Ctrl+Alt+End`
- **Process Detection**: Smart algorithm to identify the "most likely game" process
- **Process Termination**: Safely kills target processes with proper error handling

### Target Selection Algorithm
1. **Priority**: Check whitelist processes first - if any are running, kill the first match
2. **Primary**: Get foreground window → extract PID
3. **Fallback**: Find largest true fullscreen window on primary monitor
4. **Safety**: Never kill blacklisted processes (configurable via YAML)

## Key Technical Patterns

### Win32 API Usage
```rust
// Hotkey registration
RegisterHotKey(hwnd, 1, MOD_ALT | MOD_CONTROL, VK_END);

// Foreground window detection
let hwnd = GetForegroundWindow();
GetWindowThreadProcessId(hwnd, &mut pid);

// Fullscreen detection - compare to monitor bounds (not work area)
GetWindowRect(hwnd, &mut rect);
// rect == monitor.rcMonitor indicates true fullscreen
```

### Process Safety Checks
- Skip cloaked/invisible windows: `IsWindowVisible()`, `WS_EX_TOOLWINDOW`
- Configurable blacklist: Never kill processes in `blacklist` array from YAML config
- Default blacklist includes system processes: `explorer.exe`, `dwm.exe`, `csrss.exe`, `winlogon.exe`
- Enable `SeDebugPrivilege` at startup for elevated process access

### Configuration System
- Embedded default YAML config in executable using `include_str!()` 
- On first run, creates `config.yaml` in `%LOCALAPPDATA%/Endgame/`
- Config format:
```yaml
blacklist: ["explorer.exe", "dwm.exe", "csrss.exe", "winlogon.exe"]
whitelist: ["game.exe", "someapp.exe"]
```
- Whitelist processes are checked first and killed immediately if found running
- Blacklist processes are never killed regardless of other selection criteria

### Fullscreen Detection Logic
- Enumerate all top-level windows
- Compare window rect to **monitor bounds** (not work area) - games cover taskbar
- Select window with largest area that equals its monitor dimensions
- This handles both exclusive and borderless fullscreen modes

## Development Workflow

### Required Crates
- `windows`: Win32 API bindings
- `tray-icon`: System tray functionality (or manual `Shell_NotifyIconW`)
- `serde`: YAML config serialization/deserialization
- `serde_yaml`: YAML parsing support
- `simplelog`: Optional logging to `%LOCALAPPDATA%/Endgame/log.txt`

### Testing Approach
- Test with various fullscreen games/apps
- Verify hotkey works from any application context
- Test elevated process scenarios (games launched as admin)
- Validate whitelist processes are prioritized correctly
- Test blacklist safety checks prevent system process termination
- Verify config file creation and loading from embedded defaults

### Error Handling Patterns
- `OpenProcess` failures → tray balloon notification "Failed (need admin?)"
- No valid target → "No suitable target" notification
- Successful kill → temporary tooltip "Killed: foo.exe (PID 1234)"
- Config file errors → fallback to embedded defaults with warning

## Project Conventions

### Code Organization
- Single-file MVP approach for initial implementation
- Core logic in `on_hotkey()` function following the pseudocode pattern
- Separate modules for process detection, safety checks, and tray management as complexity grows

### Configuration Management
- Embed default config using `include_str!("default_config.yaml")`
- Create config directory at `%LOCALAPPDATA%/Endgame/` on first run
- Write config.yaml and log.txt to same directory
- Load user config with fallback to embedded defaults

### Logging Strategy
- Minimal logging to `%LOCALAPPDATA%/Endgame/log.txt`
- Log hotkey activations, target selection, and kill attempts
- Include process names and PIDs for debugging
- Log config loading and validation errors

### UX Principles
- Zero visible UI - tray-only operation
- Immediate feedback via tray tooltips and balloons
- Clear tooltip: "Ctrl+Alt+End → Kill foreground/fullscreen process"

## Future Enhancements (Post-MVP)
- Process tree termination (parent + children)
- Graceful shutdown attempt (`WM_CLOSE`) before hard kill
- Configurable hotkeys and per-game allowlist/denylist
- More sophisticated game detection heuristics

## Critical Implementation Notes
- Always compare window bounds to **monitor rect**, not work area for fullscreen detection
- Enable debug privileges early to handle elevated game processes
- Use `PROCESS_TERMINATE` flag for `OpenProcess`
- Handle Windows DPI scaling in coordinate comparisons
- Test thoroughly with games that use different fullscreen modes