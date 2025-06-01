#!/usr/bin/env node

import { argv, exit } from "node:process";
import { gardensCreateHandler } from "../gardens/commands/create.ts";
import { gardensDeleteHandler } from "../gardens/commands/delete.ts";
import { gardensListHandler } from "../gardens/commands/list.ts";
import { gardensWhereHandler } from "../gardens/commands/where.ts";

interface Command {
  name: string;
  description: string;
  handler?: (args: string[]) => void | Promise<void>;
}

const commands: Command[] = [
  {
    name: "create",
    description: "Create a new worktree (garden)",
    handler: gardensCreateHandler,
  },
  {
    name: "list",
    description: "List all gardens",
    handler: gardensListHandler,
  },
  {
    name: "where",
    description: "Output the path of a specific garden",
    handler: gardensWhereHandler,
  },
  {
    name: "delete",
    description: "Delete a garden (use --force for dirty gardens)",
    handler: gardensDeleteHandler,
  },
];

function printHelp() {
  console.log("Usage: garden <command> [options]\n");
  console.log("Commands:");
  for (const cmd of commands) {
    console.log(`  ${cmd.name.padEnd(20)} ${cmd.description}`);
  }
  console.log("\nThis is an alias for 'phantom garden' commands.");
}

function findCommand(cmdName: string, commands: Command[]): Command | null {
  return commands.find((cmd) => cmd.name === cmdName) || null;
}

const args = argv.slice(2);

if (args.length === 0 || args[0] === "-h" || args[0] === "--help") {
  printHelp();
  exit(0);
}

const command = findCommand(args[0], commands);

if (!command || !command.handler) {
  console.error(`Error: Unknown command '${args[0]}'\n`);
  printHelp();
  exit(1);
}

try {
  await command.handler(args.slice(1));
} catch (error) {
  console.error(
    "Error:",
    error instanceof Error ? error.message : String(error),
  );
  exit(1);
}
