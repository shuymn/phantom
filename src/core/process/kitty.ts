import type { Result } from "../types/result.ts";
import type { ProcessError } from "./errors.ts";
import { type SpawnSuccess, spawnProcess } from "./spawn.ts";

export type KittySplitDirection = "new" | "vertical" | "horizontal";

export interface KittyOptions {
  direction: KittySplitDirection;
  command: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
  windowTitle?: string;
}

export type KittySuccess = SpawnSuccess;

export async function isInsideKitty(): Promise<boolean> {
  return (
    process.env.TERM === "xterm-kitty" ||
    process.env.KITTY_WINDOW_ID !== undefined
  );
}

export async function executeKittyCommand(
  options: KittyOptions,
): Promise<Result<KittySuccess, ProcessError>> {
  const { direction, command, args, cwd, env, windowTitle } = options;

  const kittyArgs: string[] = ["@", "launch"];

  switch (direction) {
    case "new":
      kittyArgs.push("--type=tab");
      if (windowTitle) {
        kittyArgs.push(`--tab-title=${windowTitle}`);
      }
      break;
    case "vertical":
      kittyArgs.push("--location=vsplit");
      break;
    case "horizontal":
      kittyArgs.push("--location=hsplit");
      break;
  }

  if (cwd) {
    kittyArgs.push(`--cwd=${cwd}`);
  }

  // Add environment variables
  if (env) {
    for (const [key, value] of Object.entries(env)) {
      kittyArgs.push(`--env=${key}=${value}`);
    }
  }

  // Add the command and arguments
  kittyArgs.push("--");
  kittyArgs.push(command);
  if (args && args.length > 0) {
    kittyArgs.push(...args);
  }

  const result = await spawnProcess({
    command: "kitty",
    args: kittyArgs,
  });

  return result;
}
