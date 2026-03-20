# Changelog

## [0.2.2] - 2026-03-20
### Fixed
- Discord Rich Presence: fix asset image names to match Discord developer portal

## [0.1.0] - 2026-03-19
### Added
- TUI mode with live temperature display, gauge, and keyboard controls
- CLI mode with one-shot subcommands and JSON status output
- Auto device detection and selection prompt
- `Watch` command using live BLE notification stream
- Venty, Volcano Hybrid, Veazy, Crafty+ support via storz-rs 0.1
- TerminalGuard ensuring terminal is always restored on exit
- All BLE operations wrapped in 5-second timeouts
- Color-coded temperature display (green at target, amber while heating, blue cool)
- Animated connection indicator in title bar
