# git-phantom

A convenient CLI tool for managing Git worktrees with ease.

## Overview

`git phantom` is a CLI wrapper around `git worktree` that provides a more intuitive interface for creating, switching between, and managing multiple worktrees in your Git repositories.

## Installation

### Build from source

```bash
# Clone the repository
git clone https://github.com/aku11i/git-phantom.git
cd git-phantom

# Build the binary
go build -o git-phantom

# Optional: Install to your PATH
go install
```

## Usage

### Run directly with Go

```bash
# Run without building
go run main.go <command>

# Examples
go run main.go list
go run main.go help
```

### Run the built binary

```bash
# Build first
go build -o git-phantom

# Run commands
./git-phantom list
./git-phantom add /path/to/worktree
./git-phantom switch /path/to/worktree
./git-phantom remove /path/to/worktree
./git-phantom prune
```

### Use as a Git subcommand

If you install `git-phantom` to your PATH, you can use it as a Git subcommand:

```bash
# Install to PATH
go install

# Use as git subcommand
git phantom list
git phantom add feature-branch
```

## Available Commands

- `list` - List all worktrees
- `add <path>` - Create a new worktree
- `switch <path>` - Switch to a worktree (outputs cd command)
- `remove <path>` - Remove a worktree
- `prune` - Clean up non-existent worktrees
- `help` - Show help message

## Development

### Prerequisites

- Go 1.19 or later
- Git

### Building

```bash
# Build for current platform
go build

# Build with specific output name
go build -o git-phantom

# Build for different platforms
GOOS=linux GOARCH=amd64 go build -o git-phantom-linux-amd64
GOOS=darwin GOARCH=amd64 go build -o git-phantom-darwin-amd64
GOOS=windows GOARCH=amd64 go build -o git-phantom-windows-amd64.exe
```

### Testing

```bash
# Run tests (when implemented)
go test ./...
```

## License

TBD