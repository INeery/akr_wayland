# AKR (AHK Rust)

AKR is a low-latency Linux key-repeat utility with active-window filtering. The project emphasizes clear separation of responsibilities between services, predictable behavior, and flexible configuration.

## Features
- Repeat only selected keys (including the bare key and allowed modifier combinations).
- Filter by active window (substring match in the window title, case-insensitive).
- Two window detection modes:
  - polling — universal fallback (default). It tries available sources (xdotool/wmctrl/kdotool/sway) and works under both Wayland and X11/XWayland.
  - dbus (optional Cargo feature) — reactive mode via D‑Bus for KDE/GNOME. If unavailable, it automatically falls back to polling.
- Configurable repeat delay and an optional toggle key to enable/disable repetition.

## System Requirements
The application creates a virtual input device; you need appropriate permissions and the uinput module:
1) The `uinput` kernel module is loaded.
2) The user has access to `/dev/input` and `/dev/uinput` (usually via groups).

### One‑time setup
```bash
# Add your user to the required groups (re-login is required)
sudo usermod -a -G input,uinput $USER

# Load the uinput module
sudo modprobe uinput

# Load uinput at boot
echo 'uinput' | sudo tee /etc/modules-load.d/uinput.conf
# Then log out and back in so group changes take effect
```

## Build
Standard Cargo commands:
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Release optimized for size (see release-small profile in Cargo.toml)
cargo build --profile release-small
```

### Cargo features
- By default the build excludes D‑Bus (minimal dependencies):
  ```bash
  cargo build            # without dbus
  cargo test             # without dbus
  ```
- Enable the reactive D‑Bus mode only if you need it:
  ```bash
  cargo build --features dbus
  cargo test  --features dbus
  ```

## Window detection modes and behavior
Configure the mode via `window.detection_mode`:
- `dbus`: if the binary is built with `--features dbus`, reactive tracking is used (KDE/GNOME). If the feature is not compiled in or D‑Bus is unavailable in your environment, the app automatically falls back to `polling` (with a warning in logs).
- `polling`: universal polling without D‑Bus. At runtime it chooses available sources (e.g., sway → xdotool → wmctrl → kdotool) and also works for XWayland windows. No separate “xwayland” feature is required.

## Configuration
The app is configured via TOML (default `ahk.toml`). Key sections:
- logging — log level/format.
- input — the physical device selection (or auto).
- repeat — repeat parameters and optional toggle key.
- window — detection mode and window filtering.
- mappings — the list of keys with an allow‑set of modifiers.

Example:
```toml
[logging]
level = "info"
format = "pretty"
filter = "ahk_rust=info"

[input]
device_path = "auto"  # auto or a specific device path

[repeat]
repeat_delay_ms = 50
# Optional toggle key to enable/disable repetition, e.g. F12
repeat_toggle_key = "f12"

[window]
detection_mode = "polling"  # dbus | polling
polling_interval_ms = 1000
window_title_patterns = []    # Empty = any window

# Key mappings
# If a key appears in [[mappings]] then:
# - the bare key is always allowed (no modifiers), and
# - if modifiers are listed, any combination composed ONLY of these is allowed.
[[mappings]]
key = "j"
modifiers = []

[[mappings]]
key = "space"
modifiers = ["ctrl", "alt"]
```

Environment variables with the `AHK_` prefix can override configuration (see figment/env).

## Testing
Standard Rust commands:
```bash
# All tests
cargo test

# With output
cargo test -- --nocapture
```

## Quick start
1) Set up permissions (see “One‑time setup”).
2) Edit `ahk.toml` (see “Configuration”).
3) Build and run:
```bash
cargo run --release -- --config ahk.toml
```