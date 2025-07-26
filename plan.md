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
- Wine configuration presets and game templates
- Dependency management and system diagnostics
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
├── shortcuts/          # Generated .desktop files (source)
│   ├── game1.desktop
│   └── game2.desktop
├── templates/          # Game templates
│   ├── fps-game.toml
│   ├── strategy-game.toml
│   └── rpg-game.toml
├── presets/            # Wine configuration presets
│   ├── gaming.toml
│   ├── compatibility.toml
│   └── performance.toml
└── deps/               # Dependency installers cache
    ├── vcredist/
    ├── dotnet/
    └── directx/

./local/share/applications/  # Symlinked .desktop files
├── cellar-game1.desktop -> ../cellar/shortcuts/game1.desktop
└── cellar-game2.desktop -> ../cellar/shortcuts/game2.desktop
```

### Core Components

1. **Runner Management** - Download and manage Proton-GE and DXVK versions
2. **Wine Prefix Management** - Create and configure wine prefixes
3. **Game Configuration** - Per-game TOML configuration files
4. **Launch System** - Steam-style launch command processing
5. **Installation Workflow** - Manual installer support with prefix management
6. **Interactive Setup** - User-friendly configuration wizards
7. **Gamescope Integration** - Advanced display and scaling options
8. **Desktop Integration** - .desktop file generation with local symlinks
9. **Temporary Execution** - Run executables in prefixes without full setup
10. **Template System** - Pre-configured game type templates
11. **Preset System** - Wine configuration presets for different scenarios
12. **Dependency Management** - Automatic installation of Windows dependencies
13. **MangoHUD Integration** - Performance monitoring and overlay configuration
14. **System Diagnostics** - Health checking and configuration validation

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
template = "fps-game"    # Template used for creation
preset = "gaming"        # Wine configuration preset

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
create_symlink = true              # Create symlink in local applications
install_system = false             # Install to ~/.local/share/applications/
icon_path = "/path/to/icon.ico"    # Optional, extracted from exe if not provided
categories = ["Game"]
keywords = ["game", "windows"]
comment = "Windows game via Cellar"
prefix_name = true                 # Prefix with "cellar-" to avoid conflicts

[dependencies]
vcredist2019 = true
dotnet48 = false
directx = true

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
cellar prefix create <prefix-name> [--proton <version>] # Create standalone prefix
cellar prefix list                        # List all prefixes
cellar prefix remove <prefix-name>        # Remove prefix
cellar prefix run <prefix-name> <exe>     # Run executable in prefix

# Proton Prefix Creation Workflow
# When --proton is used, Cellar orchestrates the prefix creation:
# 1. Locate Runner: Find the path to the specified Proton version.
# 2. Set Environment: Construct environment variables:
#    - PROTONPATH: Path to the Proton installation.
#    - WINEPREFIX: The full path to the new prefix directory.
#    - WINEARCH: Hardcoded to `win64`.
#    - PROTON_VERB: Set to `waitforexitandrun` to trigger initialization.
# 3. Execute: Run `umu-run` with a command like `wineboot` to start creation.
```

### Runner Management
```bash
# Download and manage runners
cellar runners list                       # Show installed runners
cellar runners refresh                    # Re-scan for local runners
cellar runners available                  # Show available downloads
cellar runners install proton <version>  # Install Proton-GE version
cellar runners install dxvk <version>    # Install DXVK version
cellar runners install vkd3d <version>   # Install vkd3d-proton version

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
cellar shortcut link <game-name>          # Create symlink to local applications
cellar shortcut unlink <game-name>        # Remove symlink only
cellar shortcut sync                      # Sync all shortcuts
cellar shortcut link-all                  # Symlink all games to local applications
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

### Template System
```bash
# Template management
cellar template list                      # List available templates
cellar template create fps-game           # Create new template
cellar template edit fps-game             # Edit existing template
cellar template show fps-game             # Show template configuration
cellar add <game-name> --template fps-game
```

### Preset System
```bash
# Wine configuration presets
cellar preset list                        # List available presets
cellar preset create gaming               # Create new preset
cellar preset edit gaming                 # Edit existing preset
cellar preset apply gaming <game-name>    # Apply preset to game
cellar config <game-name> --preset gaming
```

### Dependency Management
```bash
# Dependency management
cellar deps list                          # List available dependencies
cellar deps install vcredist2019          # Install dependency globally
cellar deps install dotnet48 <game-name>  # Install for specific game
cellar deps check <game-name>             # Check game dependencies
cellar deps update                        # Update dependency cache
cellar deps remove vcredist2019           # Remove dependency
```

### MangoHUD Integration
```bash
# MangoHUD management
cellar mangohud enable <game-name>        # Enable MangoHUD for game
cellar mangohud disable <game-name>       # Disable MangoHUD for game
cellar mangohud config <game-name>        # Configure MangoHUD settings
cellar mangohud global-config             # Configure global MangoHUD
```

### System Diagnostics
```bash
# System health and diagnostics
cellar doctor                             # Run comprehensive system health check
cellar doctor <game-name>                 # Check specific game health
cellar validate <game-name>               # Validate game configuration
cellar validate --all                     # Validate all configurations
cellar fix-permissions                    # Fix file permissions issues
```

### Interactive Setup Flow

#### Main Game Addition (`cellar add -i`)
```
Cellar - Add New Game

Game name: [My Game]

Setup type:
> Add existing game (I have the game installed)
  Install new game (I have an installer)

--- If "Add existing game" ---
Game executable: [Browse/Enter path]

--- If "Install new game" ---
Installer executable: [Browse/Enter path]

Template:
> FPS Game (optimized for first-person shooters)
  Strategy Game (optimized for strategy games)
  RPG Game (optimized for RPGs)
  Custom setup

Wine Configuration:
> Use preset: Gaming (recommended)
  Use preset: Compatibility
  Use preset: Performance
  Custom configuration

Select Proton version:
> GE-Proton8-32 (installed)
  GE-Proton9-1 (installed)
  GE-Proton9-2 (available for download)

Select DXVK version:
> Use bundled DXVK
  v2.3.1 (installed)
  v2.4 (available for download)

Performance Monitoring:
> [x] Enable MangoHUD
  [x] Enable GameMode (auto-detected)

Gamescope Configuration:
> [ ] Enable gamescope

Desktop Integration:
> [x] Create application shortcut
  [x] Install to local applications menu

Dependencies:
> [x] Visual C++ Redistributable 2019
  [ ] .NET Framework 4.8
  [x] DirectX End-User Runtime

Launch options (Steam-style):
[PROTON_ENABLE_WAYLAND=1 gamemoderun %command%]

Game arguments:
[--windowed --dx11]

Wine prefix:
> Create new prefix: ./local/share/cellar/prefixes/my_game
  Use existing prefix: [Browse]
```

Game arguments:
[--windowed --dx11]

Wine prefix:
> Create new prefix: ./local/share/cellar/prefixes/my_game
  Use existing prefix: [Browse]
```

#### Installation Workflow (if installer selected)
```
Ready to install? This will:
1. Create wine prefix at: ./local/share/cellar/prefixes/my_game
2. Launch installer: /path/to/installer.exe
3. Wait for you to complete installation manually
4. Configure game executable

Continue? [Y/n]

Creating prefix...
Launching installer...

Complete the installation manually, then press enter to continue...

Installation completed!
Scanning for executables...

Found executables:
> C:\Program Files\My Game\game.exe
  C:\Program Files\My Game\launcher.exe
  Browse for different executable

Game "My Game" configured successfully!
Desktop shortcut created and linked to local applications.
Installing dependencies: Visual C++ Redistributable 2019, DirectX...
Dependencies installed successfully!
```

#### Gamescope Configuration (if enabled)
```
Gamescope Configuration:
Resolution: [1920x1080]
Refresh rate: [60] Hz

Upscaling:
> FSR
  NIS
  Linear
  Nearest
  Off

Display options:
> [x] Fullscreen
  [ ] Borderless
  [x] Steam integration
  [ ] Force grab cursor
  [ ] HDR support
  [ ] Adaptive sync
  [ ] Immediate flips
```

## Template and Preset System

### Game Templates
```toml
# fps-game.toml template
[wine_config]
esync = true
fsync = true
dxvk = true
dxvk_async = true
large_address_aware = true

[gamescope]
enabled = false
width = 1920
height = 1080
refresh_rate = 144
upscaling = "fsr"

[mangohud]
enabled = true
fps = true
gpu_stats = true
frame_timing = true

[dependencies]
vcredist2019 = true
directx = true
```

### Wine Configuration Presets
```toml
# gaming.toml preset
[wine_config]
esync = true
fsync = true
dxvk = true
dxvk_async = true

# compatibility.toml preset
[wine_config]
esync = false
fsync = false
dxvk = false
large_address_aware = true

# performance.toml preset
[wine_config]
esync = true
fsync = true
dxvk = true
dxvk_async = false  # For stability
large_address_aware = true
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

# Option B: Separate steps
cellar add "Witcher 3" --install
cellar installer "Witcher 3" "/path/to/setup.exe"
cellar setup "Witcher 3" --exe "/path/to/witcher3.exe"

# Option C: Interactive
cellar add "Witcher 3"
# Select "Install new game" in interactive mode
```

### Template and Preset Usage
```bash
# Using templates
cellar add "Counter-Strike 2" --template fps-game
cellar template apply fps-game "existing-game"

# Using presets
cellar add "Old Game" --preset compatibility
cellar preset apply gaming "Cyberpunk 2077"
```

### Dependency Management
```bash
# Install dependencies for new game
cellar add "My Game" --deps vcredist2019,directx

# Check and install missing dependencies
cellar deps check "Cyberpunk 2077"
cellar deps install dotnet48 "Cyberpunk 2077"
```

### MangoHUD Integration
```bash
# Enable MangoHUD with default settings
cellar mangohud enable "Cyberpunk 2077"

# Configure MangoHUD settings
cellar config "Cyberpunk 2077" mangohud.fps=true mangohud.gpu_stats=true
```

### System Diagnostics
```bash
# Run system health check
cellar doctor

# Check specific game
cellar doctor "Cyberpunk 2077"

# Validate all configurations
cellar validate --all
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. **Project Setup**
   - Initialize Cargo project with required dependencies
   - Set up CLI argument parsing with `clap`
   - Create basic project structure

2. **Configuration System**
   - Implement TOML serialization/deserialization
   - Create configuration validation
   - Set up directory structure management

3. **Basic Game Management**
   - Implement game creation and listing
   - Basic configuration file management
   - Simple game launching without runners

### Phase 2: Runner Management
4. **Runner Discovery and Management**
   - Implement runner discovery for local Wine and Proton installations.
   - Implement caching for discovered runners.
   - Build `cellar runners list` and `cellar runners refresh` commands.
   - Implement `cellar runners install` to download and extract runners.

5. **DXVK Integration**
   - DXVK version downloading and management.
   - Integration with Proton installations.

6. **Wine Prefix Management**
   - Implement `cellar prefix create`, including the Proton creation workflow using `umu`.
   - Implement `cellar prefix delete`, `list`, and `run` commands.

### Phase 3: Launch System
7. **umu-launcher Integration**
   - Command construction with local runners
   - Environment variable management (including custom variables and DLL overrides)
   - Steam-style launch command processing

8. **Advanced Configuration**
   - Wine option configuration (esync, fsync, etc.)
   - Environment variable management
   - Launch argument processing

### Phase 4: Installation and Desktop Features
9. **Manual Installation Workflow**
   - Implement `cellar install` to run installers within a prefix.
   - Implement desktop shortcut and symlink creation.
   - Installer launching and monitoring
   - Post-installation executable detection

10. **Desktop Integration**
    - .desktop file generation
    - Icon extraction from executables
    - Local symlink management to ./local/share/applications

### Phase 5: Enhanced Features
11. **Gamescope Integration**
    - Gamescope command construction
    - Resolution and scaling configuration
    - Display option management

12. **Interactive Setup**
    - User-friendly setup wizards
    - Configuration validation and preview
    - File browsing and path selection

### Phase 6: Template and Preset Systems
13. **Template System**
    - Pre-configured game type templates (FPS, Strategy, RPG)
    - Template creation and editing
    - Template application to games

14. **Preset System**
    - Wine configuration presets (Gaming, Compatibility, Performance)
    - Preset creation and management
    - Easy preset application

### Phase 7: Advanced Features
15. **Dependency Management**
    - Windows dependency detection and installation
    - Visual C++ Redistributables, .NET Framework, DirectX
    - Per-game dependency tracking

16. **MangoHUD Integration**
    - MangoHUD configuration management
    - Performance overlay settings
    - Custom MangoHUD config support

### Phase 8: Utilities and Diagnostics
17. **Temporary Execution**
    - Prefix-based temporary execution
    - Utility command shortcuts (winetricks, winecfg, etc.)
    - Temporary prefix management

18. **System Diagnostics**
    - Comprehensive system health checking
    - Configuration validation and repair
    - Dependency verification

### Phase 9: Polish and Testing
19. **Error Handling and Validation**
    - Comprehensive error messages
    - Configuration validation
    - Dependency checking

20. **Testing and Documentation**
    - Unit and integration tests
    - User documentation
    - Example configurations

## Dependencies

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
│   ├── template.rs        # Template system
│   ├── preset.rs          # Preset system
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
├── deps/                  # Dependency management
│   ├── mod.rs
│   ├── manager.rs         # Dependency installation
│   └── registry.rs        # Available dependencies
├── diagnostics/           # System diagnostics
│   ├── mod.rs
│   ├── doctor.rs          # Health checking
│   └── validator.rs       # Configuration validation
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
   - Generate desktop shortcuts with local symlinks
   - Interactive setup and configuration editing
   - Temporary execution of utilities in prefixes
   - Template and preset system for easy game configuration
   - Windows dependency management and installation
   - MangoHUD integration for performance monitoring
   - System diagnostics and configuration validation

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
   - Template-based setup for common game types
   - Preset-based wine configuration for different scenarios
   - Automated dependency detection and installation
   - Clear status messages and progress indicators
   - Flexible configuration options
   - Reliable game launching with gamescope and MangoHUD support

4. **Technical Requirements**
   - Local runner management (no system dependencies)
   - Portable project structure
   - Efficient symlink management for shortcuts
   - Support for both existing games and new installations
   - Temporary prefix execution for utilities
   - Template and preset management systems
   - Dependency caching and management
   - System health monitoring and validation

This plan provides a comprehensive roadmap for implementing all requested features while maintaining good software engineering practices and user experience. The redesigned CLI interface makes the tool more intuitive and supports both simple game addition and complex installation workflows, with advanced features like templates, presets, dependency management, and system diagnostics.
