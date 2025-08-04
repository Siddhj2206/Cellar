# Cellar

A wine prefix and game manager for Linux that simplifies running Windows games and applications.

## Overview

Cellar is a command-line tool designed to make managing Wine prefixes and Windows games easy on Linux.

## Features

- **Game Management**: Add, launch, list, and remove Windows games with ease
- **Wine Prefix Management**: Create and manage isolated Wine environments
- **Runner Support**: Automatic Proton-GE and DXVK installation and management (currently tested with Proton-GE only)
- **Desktop Integration**: Automatic desktop shortcut creation with icon extraction
- **Gamescope Integration**: Built-in support for Gamescope configuration

## Installation

### Prerequisites

- Rust (1.70+)
- Wine
- `umu launcher` (for Proton support)
- `wineboot` (for Wine prefix creation)
- `icoutils` (for icon extraction from executables)
- `imagemagick` (for icon processing and conversion)
- `gamemode` (optional, for performance optimization)
- `gamescope` (optional, for display/window management)
- `mangohud` (optional, for performance overlay)

### Building from Source

```bash
git clone https://github.com/Siddhj2206/Cellar.git
cd cellar
cargo build --release
```

The binary will be available at `target/release/cellar`.

**Important**: For desktop shortcuts to work properly, copy the binary to a directory in your PATH:

```bash
cp target/release/cellar ~/.local/bin/
```

## Quick Start

1. **Install a Proton runner**:
   ```bash
   cellar runners install proton GE-Proton10-10
   ```

2. **Add a game**:
   ```bash
   cellar add "My Game" --exe /path/to/game.exe
   ```

3. **Launch the game**:
   ```bash
   cellar launch "My Game"
   ```

## Commands

### Game Management

- `cellar add <name>` - Add a new game
  - `--exe <path>` - Path to existing executable
  - `--proton <version>` - Specify Proton version
  - `--prefix <name>` - Specify prefix name (defaults to game name)

- `cellar launch <name>` - Launch a game
- `cellar list` - List all configured games
- `cellar remove <name>` - Remove a game (with optional prefix cleanup)
- `cellar info <name>` - Show detailed game information


### Runner Management

- `cellar runners list` - List installed runners
- `cellar runners available` - Show available runners for download
- `cellar runners install <type> <version>` - Install a runner (proton/dxvk)
- `cellar runners remove <type> <version>` - Remove a runner
- `cellar runners refresh` - Refresh runner cache
- `cellar runners install-dxvk <version> <prefix>` - Install DXVK to specific prefix

### Prefix Management

- `cellar prefix create <name>` - Create a new Wine prefix
  - `--proton <version>` - Use specific Proton version
- `cellar prefix list` - List all prefixes
- `cellar prefix remove <name>` - Remove a prefix
- `cellar prefix run <prefix> <exe>` - Run executable in prefix
  - `--proton <version>` - Use specific Proton version

### Desktop Shortcuts

- `cellar shortcut create <name>` - Create desktop shortcut for game
- `cellar shortcut remove <name>` - Remove desktop shortcut
- `cellar shortcut sync` - Sync all desktop shortcuts
- `cellar shortcut list` - List all shortcuts
- `cellar shortcut extract-icon <name>` - Extract icon from game executable
- `cellar shortcut list-icons` - List all extracted icons

## Configuration

Games are configured using TOML files stored in `~/.local/share/cellar/configs/`. Each game has its own configuration file with settings for:

- Wine/Proton configuration (esync, fsync, DXVK)
- Gamescope settings (resolution, upscaling, refresh rate)
- Launch options and environment variables
- Desktop integration settings

Example configuration:
```toml
[game]
name = "My Game"
executable = "/path/to/game.exe"
wine_prefix = "/home/user/.local/share/cellar/prefixes/my-game"
proton_version = "GE-Proton10-10"

[wine_config]
esync = true
fsync = true
dxvk = true
dxvk_async = false

[gamescope]
enabled = false
width = 1920
height = 1080
output_width = 1920
output_height = 1080
refresh_rate = 60
upscaling = "fsr"
```

## Directory Structure

Cellar organizes files in the following structure:
```
~/.local/share/cellar/
├── configs/          # Game configuration files
├── prefixes/         # Wine prefixes
├── runners/          # Proton and DXVK installations
├── cache/            # Runner cache and temporary files
└── icons/            # Extracted game icons
```

## Dependencies

### Runtime Dependencies
- `anyhow` - Error handling
- `clap` - Command-line parsing
- `tokio` - Async runtime
- `serde` - Serialization
- `toml` - Configuration format
- `reqwest` - HTTP client for downloads
- `tar`, `zip`, `flate2` - Archive handling
- `regex` - Pattern matching
- `chrono` - Date/time handling
- `dirs` - Directory utilities

### Development Dependencies
- `tempfile` - Temporary files for testing
- `tokio-test` - Async testing utilities

## Compatibility

**Note**: This project has been primarily tested with Proton-GE runners. While other Proton versions may work, they have not been tested.

## Testing

Run the test suite:
```bash
cargo test
```

Run specific tests:
```bash
cargo test test_name
cargo test module_name
```
