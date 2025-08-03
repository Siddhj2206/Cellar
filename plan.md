# Cellar - Wine Prefix and Game Manager

A Rust-based tool for managing wine prefixes and launching games with umu-launcher, Proton-GE, and advanced configuration options.

## Overview

Cellar provides a Steam-like experience for managing Windows games on Linux through wine prefixes, with support for:
- Local Proton-GE and DXVK version management
- Steam-style launch commands with `%command%` placeholder
- Manual installer workflow with prefix management
- Interactive setup and configuration
- Gamescope and MangoHUD integration
- Desktop file generation with local symlinks
- Per-game configuration files

## Project Structure

### Directory Layout
```
./local/share/cellar/
├── runners/
│   ├── proton/
│   │   ├── GE-Proton8-32/
│   │   ├── GE-Proton9-1/
│   │   └── ...
│   └── dxvk/
│       ├── v2.3.1/
│       ├── v2.4/
│       └── ...
├── prefixes/
│   ├── game1/
│   └── game2/
├── configs/
│   ├── game1.toml
│   └── game2.toml
├── icons/              # Game icons
│   ├── game1.png
│   └── game2.ico

./local/share/applications/  # .desktop files created directly here
├── cellar-game1.desktop
└── cellar-game2.desktop
```

### Core Components

1. **Runner Management** - Download and manage Proton-GE and DXVK versions
2. **Wine Prefix Management** - Create and configure wine prefixes
3. **Game Configuration** - Per-game TOML configuration files
4. **Launch System** - Steam-style launch command processing
5. **Installation Workflow** - Manual installer support with prefix management
6. **Interactive Setup** - User-friendly configuration wizards
7. **Gamescope Integration** - Advanced display and scaling options
8. **Desktop Integration** - .desktop file generation
9. **Temporary Execution** - Run executables in prefixes without full setup
10. **MangoHUD Integration** - Performance monitoring and overlay configuration

## Configuration Format

### Game Configuration (config.toml)
```toml
[game]
name = "Game Name"
executable = "/path/to/game.exe"
wine_prefix = "./local/share/cellar/prefixes/game_name"
proton_version = "GE-Proton8-32"
dxvk_version = "v2.3.1"  # Optional, uses bundled if not specified
status = "configured"    # "configured", "installing", "installed", "incomplete"

[launch]
# Steam-style launch command with %command% placeholder
launch_options = "PROTON_ENABLE_WAYLAND=1 gamemoderun %command%"
game_args = ["--windowed", "--dx11"]

[wine_config]
esync = true
fsync = true
dxvk = true
dxvk_async = true
large_address_aware = false
wineserver_kill_timeout = 5

[dxvk]
hud = "devinfo,fps" # DXVK_HUD environment variable
# Note: DXVK_STATE_CACHE_PATH will be managed automatically by Cellar.

[gamescope]
enabled = true
width = 1920
height = 1080
refresh_rate = 60
upscaling = "fsr"  # "fsr", "nis", "linear", "nearest", "off"
fullscreen = true
borderless = false
steam_integration = true
force_grab_cursor = false
expose_wayland = false
hdr = false
adaptive_sync = false
immediate_flips = false

[mangohud]
enabled = true
fps = true
gpu_stats = true
cpu_stats = true
frame_timing = false
config_file = "~/.config/MangoHud/MangoHud.conf"  # Optional custom config

[desktop]
create_shortcut = true
icon_path = "/path/to/icon.ico"    # Optional, extracted from exe if not provided
categories = ["Game"]
keywords = ["game", "windows"]
comment = "Windows game via Cellar"
prefix_name = true                 # Prefix with "cellar-" to avoid conflicts

[installation]
# Present only if game was installed via cellar
installer_path = "/path/to/installer.exe"
install_date = "2024-01-15T10:30:00Z"
install_location = "C:\\Program Files\\Game Name"
```

## CLI Interface

### Core Game Management
```bash
# Add new games
cellar add <game-name>                    # Interactive setup for new game
cellar add <game-name> --exe <path>       # Quick add with existing executable
cellar add <game-name> --installer <path> # Add game with manual installer

# Launch games
cellar launch <game-name>                 # Launch configured game
cellar run <game-name>                    # Alias for launch

# List and manage games
cellar list                               # List all games
cellar remove <game-name>                 # Remove game
cellar info <game-name>                   # Show game configuration
cellar status                             # Show all game statuses
cellar status <game-name>                 # Show specific game status
```

### Installation Workflow
```bash
# Manual installation process
cellar add <game-name> --install          # Create prefix for installation
cellar installer <game-name> <installer>  # Run installer in existing game prefix
cellar setup <game-name> --exe <path>     # Set executable after installation
cellar setup <game-name>                  # Interactive executable selection
cellar finalize <game-name> --exe <path>  # Complete configuration after install
cellar scan <game-name>                   # Scan prefix for executables
```

### Configuration Management
```bash
# Configure games
cellar config <game-name>                 # Show current config
cellar config <game-name> edit            # Interactive config editor
cellar config <game-name> <key>=<value>   # Quick config changes

# Examples of quick config
cellar config my-game gamescope=true
cellar config my-game resolution=2560x1440
cellar config my-game proton=GE-Proton9-1
cellar config my-game esync=false dxvk=true
```

### Prefix Management
```bash
# Prefix operations
cellar prefix create <prefix-name> --proton <version> # Create standalone prefix
cellar prefix list                        # List all prefixes
cellar prefix remove <prefix-name>        # Remove prefix
cellar prefix run <prefix-name> <exe> --proton <version> # Run executable in prefix

# Proton Prefix Creation Workflow
# When --proton is used, Cellar creates prefixes using the Lutris method:
# 1. Locate Runner: Find the path to the specified Proton version.
# 2. Set Environment: Construct environment variables:
#    - WINEARCH: Set to `win64`.
#    - WINEPREFIX: The full path to the new prefix directory.
#    - WINEDLLOVERRIDES: Empty string for clean setup.
#    - WINE_MONO_CACHE_DIR: Points to Proton's mono cache.
#    - WINE_GECKO_CACHE_DIR: Points to Proton's gecko cache.
#    - PROTON_VERB: Set to `run` for prefix creation.
#    - PROTONPATH: Path to the Proton installation.
#    - GAMEID: Set to `umu-default`.
# 3. Execute: Run `umu-run createprefix` to initialize the prefix.
# 4. Marker: Create `proton_version.txt` to remember Proton version used.

# Proton Execution
# When running executables in Proton prefixes:
# - Automatically detects Proton prefixes using `proton_version.txt` marker
# - Uses `umu-run` with proper environment variables:
#   - PROTON_VERB=waitforexitandrun for main execution
#   - WINE_LARGE_ADDRESS_AWARE=1 for compatibility
#   - Proper PROTONPATH and GAMEID settings
```

### Runner Management
```bash
# Download and manage runners
cellar runners list                       # Show installed runners
cellar runners refresh                    # Re-scan for local runners
cellar runners available                  # Show available downloads
cellar runners install proton <version>  # Install Proton-GE version
cellar runners install dxvk <version>    # Install DXVK version
cellar runners remove proton <version>   # Remove/uninstall Proton-GE version
cellar runners remove dxvk <version>     # Remove/uninstall DXVK version
cellar runners install-dxvk <version> <prefix> # Install DXVK into specific prefix

# Runner Discovery Logic
# - Scans default Steam directory (~/.steam/steam/steamapps/common/)
# - Scans a local Cellar directory (~/.local/share/cellar/runners/)
# - Caches discovered runner paths for fast lookups.
```

### Desktop Integration
```bash
# Shortcut management
cellar shortcut create <game-name>        # Create desktop shortcut
cellar shortcut remove <game-name>        # Remove shortcut
cellar shortcut sync                      # Sync all shortcuts
```

### Temporary Execution
```bash
# Run executables temporarily
cellar exec <prefix-name> <executable>    # Run exe in existing prefix
cellar exec --temp <executable>           # Run exe in temporary prefix

# Utility executions
cellar winetricks <prefix-name>           # Run winetricks in prefix
cellar winecfg <prefix-name>              # Run winecfg in prefix
cellar regedit <prefix-name>              # Run regedit in prefix
```

### Maintenance Commands
```bash
# System maintenance
cellar clean                              # Remove temp files and broken symlinks
cellar check                              # Verify all configurations
cellar repair <game-name>                 # Repair broken game config
cellar fix <game-name>                    # Fix incomplete game setup
```

## Usage Examples

### Simple Game Addition
```bash
# Add existing game
cellar add "Cyberpunk 2077" --exe "/path/to/Cyberpunk2077.exe"

# Interactive setup
cellar add "Cyberpunk 2077"
```

### Installation Workflow
```bash
# Option A: All-in-one installation
cellar add "Witcher 3" --installer "/path/to/setup.exe"

# Option B: Interactive
cellar add "Witcher 3"
# Select "Install new game" in interactive mode
```

### Interactive Setup Flow

### ✅ Phase 1: Core Infrastructure (COMPLETED)
1. **Project Setup** - ✅ DONE
   - Cargo project with required dependencies (clap, serde, toml, anyhow, tokio, reqwest, etc.)
   - CLI argument parsing with `clap` derive macros
   - Modular project structure with separate modules for CLI, config, launch, runners, utils

2. **Configuration System** - ✅ DONE
   - TOML serialization/deserialization with comprehensive GameConfig structure
   - Configuration validation with dedicated validation module
   - Directory structure management through CellarDirectories utility
   - Support for all planned config sections (game, launch, wine_config, dxvk, gamescope, mangohud, desktop)

3. **Basic Game Management** - ✅ DONE
   - Game creation and listing with proper sanitized filenames
   - Configuration file management in configs/ directory
   - Game status tracking (configured, installing, installed, incomplete)
   - Game removal and info display commands

### ✅ Phase 2: Runner Management (COMPLETED)
4. **Runner Discovery and Management** - ✅ DONE
   - Async runner discovery for local Proton and DXVK installations
   - RunnerManager trait with concrete ProtonManager and DxvkManager implementations
   - `cellar runners list`, `refresh`, `available`, `install`, `remove` commands
   - GitHub API integration for downloading Proton-GE and DXVK releases
   - Local installation with tar/zip extraction support

5. **DXVK Integration** - ✅ DONE
   - DXVK version downloading and management
   - `cellar runners install-dxvk` for installing DXVK into prefixes
   - DXVK configuration options in game configs

6. **Wine Prefix Management** - ✅ DONE
   - `cellar prefix create` with Proton version support
   - `cellar prefix remove`, `list`, `run` commands
   - Prefix creation workflow with environment variable setup
   - Proton version marker files for auto-detection

### ✅ Phase 3: Launch System (COMPLETED)
7. **umu-launcher Integration** - ✅ DONE
   - Launch executor with proper environment variable management
   - Command construction for Proton/Wine execution
   - Steam-style launch command processing with `%command%` placeholder replacement
   - Full game launching system with `cellar launch` command
   - Error handling and output filtering

8. **Advanced Configuration** - ✅ DONE
   - Wine option configuration (esync, fsync, dxvk, dxvk_async, large_address_aware)
   - Environment variable management with Wine/Proton variables
   - Launch argument processing and game-specific arguments
   - DXVK configuration with HUD settings
   - GameScope and MangoHUD configuration structures

### Phase 4: Installation and Desktop Features
9. **Manual Installation Workflow**
   - Basic installer support planned but not fully implemented
   - Need to implement `cellar install` to run installers within a prefix
   - Need desktop shortcut creation
   - Need post-installation executable detection and scanning

10. **Desktop Integration** - ❌ NOT STARTED
    - .desktop file generation not implemented
    - Icon extraction from executables not implemented

11. **Interactive Setup** - ❌ NOT STARTED
    - User-friendly setup wizards not implemented
    - Configuration validation and preview not implemented
    - File browsing and path selection not implemented

### ❌ Phase 5: Enhanced Features (NOT STARTED)
12. **Gamescope Integration** - 🔄 PARTIAL
    - Gamescope configuration structure exists in config
    - Command construction and execution not implemented
    - Resolution and scaling configuration not implemented
    - Display option management not implemented

12. **MangoHUD Integration** - 🔄 PARTIAL
    - MangoHUD configuration structure exists in config
    - MangoHUD integration using 'mangohud' command wrapper implemented
    - Configuration is managed through game config files

### ❌ Phase 6: Utilities (NOT STARTED)
13. **Temporary Execution** - ❌ NOT STARTED
    - Prefix-based temporary execution not implemented
    - Utility command shortcuts (winetricks, winecfg, etc.) not implemented
    - Temporary prefix management not implemented

### ❌ Phase 7: Polish and Testing (PARTIALLY IMPLEMENTED)
14. **Error Handling and Validation** - 🔄 PARTIAL
    - Basic error handling with anyhow implemented
    - Configuration validation partially implemented
    - Need comprehensive error messages

16. **Testing and Documentation** - 🔄 PARTIAL
    - Test modules exist but may not be comprehensive
    - User documentation exists in plan.md
    - Need more example configurations and integration tests

## Current Working Features

### ✅ Fully Implemented Commands
```bash
# Game Management (Full Implementation)
cellar add <game-name> --exe <path>       # Add existing game with executable
cellar add <game-name> --proton <version> # Add game with specific Proton version
cellar add <game-name> --prefix <n>       # Add game with custom prefix name
cellar launch <game-name>                 # Launch configured game with full environment setup
cellar list                               # List all games with status information
cellar remove <game-name>                 # Remove game configuration
cellar info <game-name>                   # Show detailed game configuration
cellar status [game-name]                 # Show game status (all games or specific game)

# Runner Management (Full Implementation)
cellar runners list                       # Show installed Proton and DXVK runners
cellar runners refresh                    # Re-scan for locally installed runners
cellar runners available                  # Show available versions for download
cellar runners install proton <version>  # Download and install Proton-GE version
cellar runners install dxvk <version>    # Download and install DXVK version
cellar runners remove proton <version>   # Remove installed Proton-GE version
cellar runners remove dxvk <version>     # Remove installed DXVK version
cellar runners install-dxvk <version> <prefix> # Install DXVK into specific prefix

# Prefix Management (Full Implementation)
cellar prefix create <name> --proton <version> # Create Proton-enabled prefix
cellar prefix create <name>               # Create basic Wine prefix
cellar prefix list                        # List all managed prefixes
cellar prefix remove <name>               # Remove prefix and all contents
cellar prefix run <prefix> <exe> --proton <version> # Run executable with Proton
cellar prefix run <prefix> <exe>          # Run executable (auto-detect Proton)
```

### 🔄 Partially Implemented Features
```bash
# Interactive Setup (Structure Only)
cellar add <game-name> --interactive       # Flag exists but interactive prompts not implemented
```

### ❌ Not Yet Implemented Commands
```bash
# Configuration Management
cellar config <game-name>                 # Show current config
cellar config <game-name> edit            # Interactive config editor
cellar config <game-name> <key>=<value>   # Quick config changes

# Desktop Integration
cellar shortcut create <game-name>        # Create desktop shortcut
cellar shortcut remove <game-name>        # Remove shortcut
cellar shortcut sync                      # Sync all shortcuts

# Utilities
cellar exec <prefix> <executable>         # Temporary execution
cellar winetricks/winecfg/regedit <prefix> # Utility shortcuts
cellar clean                              # Cleanup temp files

cellar add <game-name> --installer <path> # Add game with installer workflow
```

### 🔧 Technical Implementation Details

**Game Launching System:**
- ✅ Full Steam-style launch command processing with `%command%` placeholder replacement
- ✅ Environment variable management for Wine/Proton/DXVK/MangoHUD through LaunchExecutor
- ✅ Game argument processing and launch option handling
- ✅ Error filtering and output handling for clean user experience
- ✅ GameScope integration with full command construction and --mangoapp support
- ✅ MangoHUD integration using 'mangohud' command wrapper (not environment variables)

**Configuration Management:**
- ✅ Comprehensive TOML-based configuration with all planned sections
- ✅ Game config validation with proper error handling
- ✅ Sanitized filename generation for game configs
- ✅ Directory structure management through CellarDirectories utility
- ✅ Support for all wine options (esync, fsync, dxvk, dxvk_async, large_address_aware)

**Prefix Management:**
- ✅ Proton prefix creation with proper environment variable setup
- ✅ Wine prefix creation for non-Proton scenarios
- ✅ Prefix version marker files for auto-detection
- ✅ Prefix execution with both explicit and auto-detected Proton versions
- ✅ Proper cleanup and removal of prefixes

**Runner Management:**
- ✅ Async GitHub API integration for downloading releases
- ✅ Local installation with tar/zip extraction support
- ✅ RunnerManager trait-based architecture for extensibility
- ✅ Proton-GE and DXVK version management
- ✅ Installation verification and caching

**CLI Architecture:**
- ✅ Clap-based command structure with proper subcommands
- ✅ Async runtime with Tokio for efficient I/O operations
- ✅ Comprehensive error handling with anyhow
- ✅ Modular command handling with separate command implementations

**File System Management:**
- ✅ Directory structure creation and management
- ✅ Proper file path handling across platforms
- ✅ Configuration file management with atomic operations
- ✅ Archive extraction with proper error handling

**Installation Workflow:**
- Manual installer execution within wine prefixes using existing launch infrastructure
- Interactive user prompts for installation success/failure confirmation
- Retry mechanism for failed installations
- User-provided executable path validation with full system path support
- Cleanup options for failed installations (keep prefix for troubleshooting or remove)
- Installation metadata tracking (installer path, date, method)
- Integration with existing `cellar add --installer` command

### 🎯 Immediate Next Steps for Full Functionality

**High Priority (Essential Features Missing):**
1. **Interactive Setup Implementation** - Critical for user experience
2. **Desktop Integration** - .desktop file generation
3. **Manual Installation Workflow** - **COMPLETED** - Complete installer execution and post-install setup
4. ✅ **GameScope Command Construction** - **COMPLETED** - Implement actual gamescope execution
5. ✅ **MangoHUD Environment Setup** - **COMPLETED** - Complete MangoHUD integration

**Medium Priority (Quality of Life):**
1. **Configuration Management Commands** - Interactive config editing

**Low Priority (Polish and Advanced Features):**
1. **Temporary Execution Utilities** - winetricks, winecfg shortcuts
2. **Advanced Error Recovery** - Repair and fix commands
3. **Comprehensive Testing** - Integration and unit tests
4. **Performance Optimizations** - Caching and efficiency improvements

## Implementation Roadmap

### 🚀 Phase 4: Essential Missing Features (HIGH PRIORITY)
```bash
# Focus: Complete core functionality for basic usability
Target: 2-3 weeks

1. Interactive Setup Implementation
   - Implement interactive prompts for game addition
   - File browsing and path selection
   - Configuration preview and confirmation

2. Desktop Integration
   - .desktop file generation with proper categories and metadata
   - Icon extraction from executables or default icons

3. Manual Installation Workflow - **COMPLETED**
   - Installer execution within prefixes using existing CommandBuilder infrastructure
   - User prompts for installation success/failure with retry option
   - Manual executable path input with validation
   - Cleanup options for failed installations (keep/remove prefix)
   - Installation metadata tracking with InstallationInfo

4. GameScope and MangoHUD Execution
   - ✅ Complete gamescope command construction and execution
   - ✅ MangoHUD command wrapper and --mangoapp integration
   - Integration testing with actual games

5. Configuration Management
   - Interactive config editing with prompts
   - Quick config changes via command line (key=value syntax)
   - Config validation and repair functionality
```

### 🛠️ Phase 5: Quality of Life Features (MEDIUM PRIORITY)
```bash
# Focus: Improve user experience and workflow efficiency
Target: 3-4 weeks

1. System Maintenance
   - Cleanup utilities for temporary files and broken links

2. Enhanced Installation Workflow
   - Executable scanning and selection after installation
   - Installation status tracking and recovery
   - Better installer progress monitoring

3. Improved CLI Experience
   - Better help messages and examples
   - Progress indicators for long-running operations
   - More intuitive command organization
```

### 🎯 Phase 6: Advanced Features (LOW PRIORITY)
```bash
# Focus: Advanced functionality and polish
Target: 4-6 weeks

1. Utility Command Shortcuts
   - winetricks, winecfg, regedit shortcuts for prefixes
   - Temporary execution environment
   - Wine utility integration

2. Advanced Diagnostics
   - Comprehensive system health checking
   - Performance monitoring and optimization suggestions
   - Automated troubleshooting and problem resolution

3. Polish and Optimization
   - Comprehensive error handling and user feedback
   - Performance optimizations and caching improvements
   - Advanced logging and debugging capabilities
```

### 🧪 Phase 7: Testing and Documentation (ONGOING)
```bash
# Focus: Reliability and user documentation
Target: Ongoing throughout development

1. Comprehensive Testing
   - Unit tests for all major components
   - Integration tests with real games and scenarios
   - Edge case testing and error condition handling

2. User Documentation
   - Complete user manual with examples
   - Configuration guides and best practices
   - Troubleshooting and FAQ documentation

3. Code Quality
   - Code review and refactoring
   - Performance profiling and optimization
   - Security review and hardening
```

## External Dependencies

### Required Crates
```toml
[dependencies]
clap = "4.0"                    # CLI argument parsing
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"                    # Configuration serialization
anyhow = "1.0"                  # Error handling
tokio = { version = "1.0", features = ["full"] }  # Async runtime
reqwest = { version = "0.11", features = ["json"] }  # HTTP client
dirs = "5.0"                    # Directory utilities
inquire = "0.7"                 # Interactive prompts
console = "0.15"                # Terminal styling
indicatif = "0.17"              # Progress bars
sha2 = "0.10"                   # Checksum verification
zip = "0.6"                     # Archive extraction
tar = "0.4"                     # Archive extraction
flate2 = "1.0"                  # Compression support
```

### Runtime Dependencies
- **`umu`:** Required for launching games with Proton. Users will need to have this installed and available in their `PATH`.

## Launch Command Construction

### Without Gamescope
```bash
WINEESYNC=1 WINEFSYNC=1 DXVK_ASYNC=1 PROTON_ENABLE_WAYLAND=1 gamemoderun umu-run --proton /path/to/proton --prefix /path/to/prefix /path/to/game.exe --windowed --dx11
```

### With MangoHUD
```bash
MANGOHUD=1 WINEESYNC=1 WINEFSYNC=1 DXVK_ASYNC=1 PROTON_ENABLE_WAYLAND=1 gamemoderun umu-run --proton /path/to/proton --prefix /path/to/prefix /path/to/game.exe --windowed --dx11
```

### With Both Gamescope and MangoHUD
```bash
MANGOHUD=1 WINEESYNC=1 WINEFSYNC=1 DXVK_ASYNC=1 PROTON_ENABLE_WAYLAND=1 gamemoderun gamescope -w 1920 -h 1080 -r 60 -U -f -- umu-run --proton /path/to/proton --prefix /path/to/prefix /path/to/game.exe --windowed --dx11
```

### Steam-style Command Processing
The `%command%` placeholder gets replaced with the actual umu-run command:
- Input: `PROTON_ENABLE_WAYLAND=1 gamemoderun %command%`
- Output: `PROTON_ENABLE_WAYLAND=1 gamemoderun umu-run --proton ... game.exe`

## File Structure

### Source Code Organization
```
src/
├── main.rs                 # CLI entry point
├── cli/                    # Command line interface
│   ├── mod.rs
│   ├── commands.rs         # Command implementations
│   └── interactive.rs      # Interactive setup
├── config/                 # Configuration management
│   ├── mod.rs
│   ├── game.rs            # Game configuration
│   └── validation.rs      # Config validation
├── runners/               # Runner management
│   ├── mod.rs
│   ├── proton.rs          # Proton-GE handling
│   └── dxvk.rs            # DXVK handling
├── launch/                # Game launching
│   ├── mod.rs
│   ├── command.rs         # Command construction
│   ├── gamescope.rs       # Gamescope integration
│   └── mangohud.rs        # MangoHUD integration
├── prefix/                # Wine prefix management
│   ├── mod.rs
│   └── manager.rs
├── desktop/               # Desktop integration
│   ├── mod.rs
│   ├── shortcut.rs        # .desktop file generation
│   └── icon.rs            # Icon extraction and management
├── install/               # Installation workflow
│   ├── mod.rs
│   └── installer.rs       # Manual installer support
└── utils/                 # Utility functions
    ├── mod.rs
    ├── download.rs        # Download utilities
    ├── fs.rs              # File system utilities
    └── exec.rs            # Process execution helpers
```

## Success Criteria

1. **Functional Requirements**
   - Create and manage wine prefixes
   - Download and install Proton-GE versions locally
   - Launch games through umu-launcher with custom configurations
   - Support Steam-style launch commands with `%command%`
   - Handle manual game installation workflow
   - Generate desktop shortcuts
   - Interactive setup and configuration editing
   - Temporary execution of utilities in prefixes
   - MangoHUD integration for performance monitoring

2. **Quality Requirements**
   - Comprehensive error handling and user feedback
   - Clean, maintainable code structure
   - Efficient download and installation processes
   - Intuitive CLI interface with logical command grouping
   - Robust configuration validation and health checking

3. **User Experience**
   - Simple game setup process with clear workflows
   - Manual installer support with guided process
   - Desktop integration with application menu shortcuts
   - Clear status messages and progress indicators
   - Flexible configuration options
   - Reliable game launching with gamescope and MangoHUD support

4. **Technical Requirements**
   - Local runner management
   - Portable project structure
   - Efficient shortcut management
   - Support for both existing games and new installations
   - Temporary prefix execution for utilities

This plan provides a comprehensive roadmap for implementing all requested features while maintaining good software engineering practices and user experience. The redesigned CLI interface makes the tool more intuitive and supports both simple game addition and complex installation workflows.
