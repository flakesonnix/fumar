# fumar

**Control your Storz & Bickel vaporizer from the terminal.**

[![crates.io](https://img.shields.io/crates/v/fumar)](https://crates.io/crates/fumar)
[![docs.rs](https://img.shields.io/docsrs/storz-rs)](https://docs.rs/storz-rs)
[![CI](https://github.com/flakesonnix/fumar/actions/workflows/ci.yml/badge.svg)](https://github.com/flakesonnix/fumar/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A TUI, CLI, and GUI client for Storz & Bickel vaporizers, built on [storz-rs](https://github.com/flakesonnix/storz-rs). Supports live temperature monitoring, heater control, and pump control (Volcano only) over Bluetooth Low Energy.

## Install

```bash
cargo install fumar
```

Or build from source with optional features:

```bash
git clone https://github.com/flakesonnix/fumar
cd fumar

# Basic build (TUI + CLI)
cargo build --release

# With GTK4 GUI
cargo build --release --features gui

# With Discord Rich Presence
cargo build --release --features discord

# All features
cargo build --release --features gui,discord

./target/release/fumar --help
```

## Usage

### TUI mode

Launches automatically when stdout is a TTY and no subcommand is given:

```bash
fumar
```

Or force TUI mode:

```bash
fumar --tui
```

### CLI mode

```bash
fumar --cli temp          # Show current and target temperature
fumar --cli status        # JSON status dump
fumar --cli set-temp 185  # Set target to 185°C
fumar --cli heat-on       # Turn heater on
fumar --cli heat-off      # Turn heater off
fumar --cli pump-on       # Turn pump on (Volcano only)
fumar --cli watch         # Stream live updates (Ctrl+C to stop)
```

### GUI mode

Requires the `gui` feature (GTK4):

```bash
fumar --gui
```

GTK4 GUI with temperature display, slider, heater/pump buttons, and BLE scan overlay.

### Discord Rich Presence

Requires the `discord` feature:

```bash
fumar --discord
fumar --tui --discord
fumar --gui --discord
```

Shows device model, current/target temperature, and heater/pump state in your Discord profile.

## TUI keybindings

| Key | Action |
|-----|--------|
| `q`, `Esc`, `Ctrl+C` | Quit |
| `↑` / `k` | Increase target +1°C |
| `↓` / `j` | Decrease target -1°C |
| `K` | Increase target +5°C |
| `J` | Decrease target -5°C |
| `h` / `H` | Toggle heater |
| `p` / `P` | Toggle pump (Volcano only) |
| `r` / `R` | Force state refresh |
| `c` / `C` | Reconnect to device |
| `s` / `S` | Toggle settings |

## Device support

| Device | Status |
|--------|--------|
| Venty | ✅ |
| Volcano Hybrid | ✅ |
| Veazy | 🔬 (same protocol as Venty) |
| Crafty+ | 🔬 |

## Linux BLE permissions

BlueZ needs permission to start BLE scans.

**Arch Linux** (no `bluetooth` group, use polkit):

```bash
sudo tee /etc/polkit-1/rules.d/50-bluetooth.rules << 'POLKIT'
polkit.addRule(function(action, subject) {
    if (action.id === "org.bluez.Adapter.StartDiscovery" ||
        action.id === "org.bluez.Adapter.SetDiscoveryFilter") {
        return polkit.Result.YES;
    }
});
POLKIT
```

**Debian/Ubuntu/Fedora** (add yourself to the `bluetooth` group):

```bash
sudo usermod -aG bluetooth $USER
```

Log out and back in after.

## License

MIT. Not affiliated with Storz & Bickel GmbH.
