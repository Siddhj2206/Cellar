# Manual Installation Workflow

This document details the complete manual installation workflow for Cellar, allowing users to install Windows games using installers within wine prefixes.

## Overview

The manual installation workflow allows users to:
1. Create a wine prefix for a new game
2. Run a Windows installer within that prefix
3. Configure the game executable after installation
4. Handle installation failures with retry and cleanup options

## Command Usage

```bash
cellar add "Game Name" --installer "path/to/installer.exe"
```

## Detailed Workflow

### Step 1: Initial Setup (Already Exists)
- Validate installer path exists
- Create sanitized game name for prefix
- Determine Proton version (user-specified or latest available)

### Step 2: Prefix Creation (Already Exists)
```
Creating wine prefix for: Game Name
Using Proton version: GE-Proton9-1
Creating prefix: ~/.local/share/cellar/prefixes/game_name
Initializing prefix...
Successfully created prefix: game_name
```

### Step 3: Installer Execution
```
Running installer: /path/to/installer.exe
Launching installer in prefix 'game_name'...
```

**Requirements:**
- Installer runs with **visible output** (no silent mode)
- User can see installer GUI and interact with it
- Cellar waits for installer process to complete
- Status shows "Running installer..." during execution

### Step 4: Post-Installation Confirmation
After installer closes:
```
Installation completed successfully? [Y/n]:
```

**If User answers Yes (Y):**
- Continue to Step 5 (Executable Path Input)

**If User answers No (n):**
- Go to Step 6 (Retry/Cleanup Options)

### Step 5: Executable Path Input
```
Please enter the path to the game executable:
```

**Input Options:**
- Windows path: `C:\Program Files\Game Name\game.exe`
- Unix path: `/home/user/.local/share/cellar/prefixes/game_name/drive_c/Program Files/Game Name/game.exe`

**Validation:**
- Check if provided path exists within the prefix
- Convert Windows paths to Unix equivalents
- Verify file is executable

**On Success:**
```
Game "Game Name" configured successfully!
  Executable: C:\Program Files\Game Name\game.exe
  Wine Prefix: ~/.local/share/cellar/prefixes/game_name
  Proton Version: GE-Proton9-1
```

### Step 6: Retry/Cleanup Options (Failed Installation)
```
Do you want to retry the installation? [Y/n]:
```

**If User answers Yes (Y):**
- Return to Step 3 (run installer again)
- Keep existing prefix

**If User answers No (n):**
```
Do you want to delete the prefix? [Y/n]:
```

- **Yes**: Remove prefix and exit with error
- **No**: Keep prefix and exit with error

## Implementation Details

### Game Configuration
After successful installation, create game config with:

```toml
[game]
name = "Game Name"
executable = "C:\\Program Files\\Game Name\\game.exe"
wine_prefix = "/home/user/.local/share/cellar/prefixes/game_name"
proton_version = "GE-Proton9-1"
```

### Error Handling

#### Installer Execution Failures
- Show clear error messages
- Offer retry option
- Provide cleanup option

#### Invalid Executable Paths
- Validate path exists in prefix
- Provide helpful error messages
- Allow user to re-enter path

#### Prefix Creation Failures
- Show Proton-related errors clearly
- Suggest installing required Proton version
- Exit gracefully with helpful messages

## User Experience Examples

### Successful Installation
```bash
$ cellar add "Cyberpunk 2077" --installer "/home/user/Downloads/cyberpunk_installer.exe"

Creating wine prefix for: Cyberpunk 2077
Using Proton version: GE-Proton9-1
Creating prefix: cyberpunk_2077
Initializing prefix...
Successfully created prefix: cyberpunk_2077

Running installer: /home/user/Downloads/cyberpunk_installer.exe
Launching installer in prefix 'cyberpunk_2077'...

[Installer GUI appears and user completes installation]

Installation completed successfully? [Y/n]: Y

Please enter the path to the game executable: C:\Program Files\CD Projekt RED\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe

Game "Cyberpunk 2077" configured successfully!
  Executable: C:\Program Files\CD Projekt RED\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe
  Wine Prefix: /home/user/.local/share/cellar/prefixes/cyberpunk_2077
  Proton Version: GE-Proton9-1
```

### Failed Installation with Retry
```bash
$ cellar add "Old Game" --installer "/home/user/Downloads/old_game_setup.exe"

[... prefix creation ...]

Running installer: /home/user/Downloads/old_game_setup.exe
Launching installer in prefix 'old_game'...

[Installer fails or user encounters issues]

Installation completed successfully? [Y/n]: n

Do you want to retry the installation? [Y/n]: Y

Running installer: /home/user/Downloads/old_game_setup.exe
Launching installer in prefix 'old_game'...

[User fixes issue and completes installation]

Installation completed successfully? [Y/n]: Y

Please enter the path to the game executable: C:\Program Files\Old Game\game.exe

Game "Old Game" configured successfully!
```

### Failed Installation with Cleanup
```bash
$ cellar add "Broken Game" --installer "/home/user/Downloads/broken_installer.exe"

[... prefix creation and failed installation ...]

Installation completed successfully? [Y/n]: n

Do you want to retry the installation? [Y/n]: n

Do you want to delete the prefix? [Y/n]: Y

Removing prefix: broken_game
Installation cancelled.
```

## Integration Points

### Existing Commands
- Leverages existing `cellar prefix create` functionality
- Leverages exisitng `cellar prefix run` functionality
- Uses existing GameLauncher and CommandBuilder infrastructure
- Integrates with current game configuration system

This workflow provides a complete, user-friendly manual installation experience that handles common failure scenarios while maintaining simplicity and clarity.
