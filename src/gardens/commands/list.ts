import { exec } from "node:child_process";
import { access, readdir } from "node:fs/promises";
import { join } from "node:path";
import { promisify } from "node:util";
import { getGitRoot } from "../../git/libs/get-git-root.ts";

const execAsync = promisify(exec);

export interface GardenInfo {
  name: string;
  branch: string;
  status: "clean" | "dirty";
  changedFiles?: number;
}

export async function listGardens(): Promise<{
  success: boolean;
  message?: string;
  gardens?: GardenInfo[];
}> {
  try {
    const gitRoot = await getGitRoot();
    const gardensPath = join(gitRoot, ".git", "phantom", "gardens");

    // Check if gardens directory exists
    try {
      await access(gardensPath);
    } catch {
      return {
        success: true,
        gardens: [],
        message: "No gardens found (gardens directory doesn't exist)",
      };
    }

    // Read gardens directory
    let gardenNames: string[];
    try {
      const entries = await readdir(gardensPath);
      // Filter entries to only include directories
      const validEntries = await Promise.all(
        entries.map(async (entry) => {
          try {
            const entryPath = join(gardensPath, entry);
            await access(entryPath);
            return entry;
          } catch {
            return null;
          }
        }),
      );
      gardenNames = validEntries.filter(
        (entry): entry is string => entry !== null,
      );
    } catch {
      return {
        success: true,
        gardens: [],
        message: "No gardens found (unable to read gardens directory)",
      };
    }

    if (gardenNames.length === 0) {
      return {
        success: true,
        gardens: [],
        message: "No gardens found",
      };
    }

    // Get detailed information for each garden
    const gardens: GardenInfo[] = await Promise.all(
      gardenNames.map(async (name) => {
        const gardenPath = join(gardensPath, name);

        // Get current branch
        let branch = "unknown";
        try {
          const { stdout } = await execAsync("git branch --show-current", {
            cwd: gardenPath,
          });
          branch = stdout.trim() || "detached HEAD";
        } catch {
          branch = "unknown";
        }

        // Get working directory status
        let status: "clean" | "dirty" = "clean";
        let changedFiles: number | undefined;
        try {
          const { stdout } = await execAsync("git status --porcelain", {
            cwd: gardenPath,
          });
          const changes = stdout.trim();
          if (changes) {
            status = "dirty";
            changedFiles = changes.split("\n").length;
          }
        } catch {
          // If git status fails, assume unknown status
          status = "clean";
        }

        return {
          name,
          branch,
          status,
          changedFiles,
        };
      }),
    );

    return {
      success: true,
      gardens,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return {
      success: false,
      message: `Error listing gardens: ${errorMessage}`,
    };
  }
}

export async function gardensListHandler(): Promise<void> {
  const result = await listGardens();

  if (!result.success) {
    console.error(result.message);
    return;
  }

  if (!result.gardens || result.gardens.length === 0) {
    console.log(result.message || "No gardens found");
    return;
  }

  console.log("Gardens:");
  for (const garden of result.gardens) {
    const statusText =
      garden.status === "clean"
        ? "[clean]"
        : `[dirty: ${garden.changedFiles} files]`;

    console.log(
      `  ${garden.name.padEnd(20)} (branch: ${garden.branch.padEnd(20)}) ${statusText}`,
    );
  }

  console.log(`\nTotal: ${result.gardens.length} gardens`);
}
