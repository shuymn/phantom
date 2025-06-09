# Phantom Configuration

## Table of Contents

- [Configuration File](#configuration-file)
- [Configuration Options](#configuration-options)
  - [postCreate.copyFiles](#postcreatecopyfiles)
  - [postCreate.commands](#postcreatecommands)
  - [defaultMultiplexer](#defaultmultiplexer)

Phantom supports configuration through a `phantom.config.json` file in your repository root. This allows you to define files to be automatically copied and commands to be executed when creating new worktrees.

## Configuration File

Create a `phantom.config.json` file in your repository root:

```json
{
  "postCreate": {
    "copyFiles": [
      ".env",
      ".env.local",
      "config/local.json"
    ],
    "commands": [
      "pnpm install",
      "pnpm build"
    ]
  },
  "defaultMultiplexer": "tmux"
}
```

## Configuration Options

### postCreate.copyFiles

An array of file paths to automatically copy from the current worktree to newly created worktrees.

**Use Cases:**
- Environment configuration files (`.env`, `.env.local`)
- Local development settings
- Secret files that are gitignored
- Database configuration files
- API keys and certificates

**Example:**
```json
{
  "postCreate": {
    "copyFiles": [
      ".env",
      ".env.local",
      "config/database.local.yml"
    ]
  }
}
```

**Notes:**
- Paths are relative to the repository root
- Currently, glob patterns are not supported
- Files must exist in the source worktree
- Non-existent files are silently skipped
- Can be overridden with `--copy-file` command line options

### postCreate.commands

An array of commands to execute after creating a new worktree.

**Use Cases:**
- Installing dependencies
- Building the project
- Setting up the development environment
- Running database migrations
- Generating configuration files

**Example:**
```json
{
  "postCreate": {
    "commands": [
      "pnpm install",
      "pnpm db:migrate",
      "pnpm db:seed"
    ]
  }
}
```

**Notes:**
- Commands are executed in order
- Execution stops on the first failed command
- Commands run in the new worktree's directory
- Output is displayed in real-time

### defaultMultiplexer

Specifies the default terminal multiplexer to use when no explicit option is provided. This setting affects how shell and exec commands behave when using terminal splitting features.

**Values:**
- `"tmux"` - Use tmux as the default multiplexer
- `"kitty"` - Use kitty as the default multiplexer
- `"none"` - Disable automatic multiplexer usage (default)

**Example:**
```json
{
  "defaultMultiplexer": "kitty"
}
```

**Notes:**
- This setting only provides a default; command-line options override it
- Users must still be inside the appropriate environment (tmux session or kitty terminal)
- When set to "none", terminal splitting features are disabled by default

