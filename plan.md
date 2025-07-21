# Cellar - Wine Prefix and Game Manager

A Rust-based tool for managing wine prefixes and launching games with umu-launcher, Proton-GE, and advanced configuration options.

## Overview

Cellar provides a Steam-like experience for managing Windows games on Linux through wine prefixes, with support for:
- Local Proton-GE and DXVK version management
- Steam-style launch commands with `%command%` placeholder
- Manual installer workflow with prefix management
- Interactive setup and configuration
- Gamescope integration
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
└── shortcuts/          # Generated .desktop files (source)
    ├── game1.desktop
    └── game2.desktop

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
wineserver_kill_timeout = 15

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

[desktop]
create_shortcut = true
create_symlink = true              # Create symlink in local applications
install_system = false             # Install to ~/.local/share/applications/
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
cellar prefix create <prefix-name>        # Create standalone prefix
cellar prefix list                        # List all prefixes
cellar prefix remove <prefix-name>        # Remove prefix
cellar prefix run <prefix-name> <exe>     # Run executable in prefix
```

### Runner Management
```bash
# Download and manage runners
cellar runners list                       # Show installed runners
cellar runners available                  # Show available downloads
cellar runners install proton <version>  # Install Proton-GE version
cellar runners install dxvk <version>    # Install DXVK version
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

### Interactive Setup Flow

#### Main Setup Wizard (`cellar create -i`)
```
Cellar Interactive Setup

Game name: [My Game]
Executable path: [Browse/Enter path]

Select Proton version:
> GE-Proton8-32 (installed)
  GE-Proton9-1 (installed)
  GE-Proton9-2 (available for download)

Select DXVK version:
> Use bundled DXVK
  v2.3.1 (installed)
  v2.4 (available for download)

Wine Configuration:
> [x] Enable esync
  [x] Enable fsync  
  [x] Enable DXVK
  [x] Enable DXVK async
  [ ] Large address aware

Gamescope Configuration:
> [ ] Enable gamescope

Launch options (Steam-style):
[PROTON_ENABLE_WAYLAND=1 gamemoderun %command%]

Game arguments:
[--windowed --dx11]

Wine prefix:
> Create new prefix: ./local/share/cellar/prefixes/my_game
  Use existing prefix: [Browse]
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
4. **Proton-GE Integration**
   - GitHub API integration for release fetching
   - Download and extraction functionality
   - Local installation management

5. **DXVK Integration**
   - DXVK version downloading and management
   - Integration with Proton installations

6. **Wine Prefix Management**
   - Wine prefix creation and validation
   - Prefix configuration and management

### Phase 3: Launch System
7. **umu-launcher Integration**
   - Command construction with local runners
   - Environment variable management
   - Steam-style launch command processing

8. **Advanced Configuration**
   - Wine option configuration (esync, fsync, etc.)
   - Environment variable management
   - Launch argument processing

### Phase 4: Enhanced Features
9. **Gamescope Integration**
   - Gamescope command construction
   - Resolution and scaling configuration
   - Display option management

10. **Interactive Setup**
    - User-friendly setup wizards
    - Configuration validation and preview
    - File browsing and path selection

### Phase 5: Polish and Testing
11. **Error Handling and Validation**
    - Comprehensive error messages
    - Configuration validation
    - Dependency checking

12. **Testing and Documentation**
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

## Launch Command Construction

### Without Gamescope
```bash
WINEESYNC=1 WINEFSYNC=1 DXVK_ASYNC=1 PROTON_ENABLE_WAYLAND=1 gamemoderun umu-run --proton ./local/share/cellar/runners/proton/GE-Proton8-32 --prefix ./local/share/cellar/prefixes/game_name /path/to/game.exe --windowed --dx11
```

### With Gamescope
```bash
WINEESYNC=1 WINEFSYNC=1 DXVK_ASYNC=1 PROTON_ENABLE_WAYLAND=1 gamemoderun gamescope -w 1920 -h 1080 -r 60 -U -f -- umu-run --proton ./local/share/cellar/runners/proton/GE-Proton8-32 --prefix ./local/share/cellar/prefixes/game_name /path/to/game.exe --windowed --dx11
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
│   └── gamescope.rs       # Gamescope integration
├── prefix/                # Wine prefix management
│   ├── mod.rs
│   └── manager.rs
└── utils/                 # Utility functions
    ├── mod.rs
    ├── download.rs        # Download utilities
    └── fs.rs              # File system utilities
```

## Success Criteria

1. **Functional Requirements**
   - Create and manage wine prefixes
   - Download and install Proton-GE versions locally
   - Launch games through umu-launcher with custom configurations
   - Support Steam-style launch commands with `%command%`
   - Interactive setup and configuration editing

2. **Quality Requirements**
   - Comprehensive error handling and user feedback
   - Clean, maintainable code structure
   - Efficient download and installation processes
   - Intuitive CLI interface

3. **User Experience**
   - Simple game setup process
   - Clear status messages and progress indicators
   - Flexible configuration options
   - Reliable game launching

This plan provides a comprehensive roadmap for implementing all requested features while maintaining good software engineering practices and user experience.