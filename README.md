# AKR (AHK Rust) Development Guidelines

This document provides essential information for developers working on the AKR project, a modern key repetition utility for Linux written in Rust.

## Build and Configuration Instructions

### System Requirements

The application requires access to input devices and the ability to create virtual input devices:

1. The `uinput` kernel module must be loaded
2. The user must have appropriate permissions to access `/dev/input` and `/dev/uinput`

### Setup Commands

Run these commands to set up the required permissions:

```bash
# Add your user to the necessary groups
sudo usermod -a -G input,uinput $USER

# Load the uinput module
sudo modprobe uinput

# Configure uinput to load at boot
echo 'uinput' | sudo tee /etc/modules-load.d/uinput.conf

# Log out and log back in for group changes to take effect
```

### Building the Project

The project uses standard Cargo build commands:

```bash
# Development build
cargo build

# Release build
cargo build --release

# Build with optimized size (as configured in Cargo.toml)
cargo build --profile release-small
```

### Configuration

The application is configured through a TOML file (default: `ahk.toml`). Configuration includes:

- Logging settings
- Input device configuration
- Window detection settings
- Performance tuning
- Key mappings

Example configuration:

```toml
[logging]
level = "info"
format = "pretty"
filter = "ahk_rust=info"

[input]
repeat_delay_ms = 50
device_path = "auto"  # auto or path to device

[window]
detection_mode = "dbus"  # dbus, polling
polling_interval_ms = 1000
window_title_patterns = []  # Empty means all windows

[performance]
enable_metrics = false
metrics_port = 9090
channel_buffer_size = 1000

# Key mappings
[[mappings]]
key = "j"
modifiers = []

[[mappings]]
key = "space"
modifiers = ["ctrl"]
```

Configuration can also be provided through environment variables with the `AHK_` prefix.

## Testing Information

### Running Tests

The project uses standard Rust testing infrastructure. Tests can be run with:

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test services::keycode_map

# Run a specific test
cargo test services::keycode_map::tests::test_basic_key_mapping

# Run tests with output
cargo test -- --nocapture
```