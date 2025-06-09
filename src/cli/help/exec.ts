import type { CommandHelp } from "../help.ts";

export const execHelp: CommandHelp = {
  name: "exec",
  description: "Execute a command in a worktree directory",
  usage: "phantom exec [options] <worktree-name> <command> [args...]",
  options: [
    {
      name: "--fzf",
      type: "boolean",
      description: "Use fzf for interactive worktree selection",
    },
    {
      name: "--tmux, -t",
      type: "boolean",
      description: "Execute command in new tmux window",
    },
    {
      name: "--tmux-vertical, --tmux-v",
      type: "boolean",
      description: "Execute command in vertical split pane (tmux)",
    },
    {
      name: "--tmux-horizontal, --tmux-h",
      type: "boolean",
      description: "Execute command in horizontal split pane (tmux)",
    },
    {
      name: "--kitty, -k",
      type: "boolean",
      description: "Execute command in new kitty tab",
    },
    {
      name: "--kitty-vertical, --kitty-v",
      type: "boolean",
      description: "Execute command in vertical split (kitty)",
    },
    {
      name: "--kitty-horizontal, --kitty-h",
      type: "boolean",
      description: "Execute command in horizontal split (kitty)",
    },
  ],
  examples: [
    {
      description: "Run npm test in a worktree",
      command: "phantom exec feature-auth npm test",
    },
    {
      description: "Check git status in a worktree",
      command: "phantom exec bugfix-123 git status",
    },
    {
      description: "Run a complex command with arguments",
      command: "phantom exec staging npm run build -- --production",
    },
    {
      description: "Execute with interactive selection",
      command: "phantom exec --fzf npm run dev",
    },
    {
      description: "Run dev server in new tmux window",
      command: "phantom exec --tmux feature-auth npm run dev",
    },
    {
      description: "Run tests in vertical split pane",
      command: "phantom exec --tmux-v feature-auth npm test",
    },
    {
      description: "Run dev server in new kitty tab",
      command: "phantom exec --kitty feature-auth npm run dev",
    },
    {
      description: "Run tests in vertical kitty split",
      command: "phantom exec --kitty-v feature-auth npm test",
    },
    {
      description: "Interactive selection with tmux",
      command: "phantom exec --fzf --tmux npm run dev",
    },
  ],
  notes: [
    "The command is executed with the worktree directory as the working directory",
    "All arguments after the worktree name are passed to the command",
    "The exit code of the executed command is preserved",
    "With --fzf, select the worktree interactively before executing the command",
    "Tmux options require being inside a tmux session",
    "Kitty options require being inside a kitty terminal",
    "Tmux and kitty options cannot be used together",
  ],
};
