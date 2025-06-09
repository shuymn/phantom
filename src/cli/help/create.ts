import type { CommandHelp } from "../help.ts";

export const createHelp: CommandHelp = {
  name: "create",
  description: "Create a new Git worktree",
  usage: "phantom create <name> [options]",
  options: [
    {
      name: "shell",
      short: "s",
      type: "boolean",
      description:
        "Open an interactive shell in the new worktree after creation",
    },
    {
      name: "exec",
      short: "x",
      type: "string",
      description: "Execute a command in the new worktree after creation",
      example: "--exec 'npm install'",
    },
    {
      name: "tmux",
      short: "t",
      type: "boolean",
      description:
        "Open the worktree in a new tmux window (requires being inside tmux)",
    },
    {
      name: "tmux-vertical",
      type: "boolean",
      description:
        "Open the worktree in a vertical tmux pane (requires being inside tmux)",
    },
    {
      name: "tmux-horizontal",
      type: "boolean",
      description:
        "Open the worktree in a horizontal tmux pane (requires being inside tmux)",
    },
    {
      name: "kitty",
      short: "k",
      type: "boolean",
      description:
        "Open the worktree in a new kitty tab (requires being inside kitty)",
    },
    {
      name: "kitty-vertical",
      type: "boolean",
      description:
        "Open the worktree in a vertical kitty split (requires being inside kitty)",
    },
    {
      name: "kitty-horizontal",
      type: "boolean",
      description:
        "Open the worktree in a horizontal kitty split (requires being inside kitty)",
    },
    {
      name: "copy-file",
      type: "string",
      multiple: true,
      description:
        "Copy specified files from the current worktree to the new one. Can be used multiple times",
      example: "--copy-file .env --copy-file config.local.json",
    },
  ],
  examples: [
    {
      description: "Create a new worktree named 'feature-auth'",
      command: "phantom create feature-auth",
    },
    {
      description: "Create a worktree and open a shell in it",
      command: "phantom create bugfix-123 --shell",
    },
    {
      description: "Create a worktree and run npm install",
      command: "phantom create new-feature --exec 'npm install'",
    },
    {
      description: "Create a worktree in a new tmux window",
      command: "phantom create experiment --tmux",
    },
    {
      description: "Create a worktree in a new kitty tab",
      command: "phantom create experiment --kitty",
    },
    {
      description: "Create a worktree and copy environment files",
      command:
        "phantom create staging --copy-file .env --copy-file database.yml",
    },
  ],
  notes: [
    "The worktree name will be used as the branch name",
    "Only one of --shell, --exec, --tmux, or --kitty options can be used at a time",
    "File copying can also be configured in phantom.config.json",
  ],
};
