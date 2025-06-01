import { exec } from "node:child_process";
import { access } from "node:fs/promises";
import { join } from "node:path";
import { exit } from "node:process";
import { promisify } from "node:util";
import { getGitRoot } from "../../git/libs/get-git-root.ts";

const execAsync = promisify(exec);

export async function deleteGarden(
  name: string,
  options: { force?: boolean } = {},
): Promise<{
  success: boolean;
  message: string;
  hasUncommittedChanges?: boolean;
  changedFiles?: number;
}> {
  if (!name) {
    return { success: false, message: "Error: garden name required" };
  }

  const { force = false } = options;

  try {
    const gitRoot = await getGitRoot();
    const gardensPath = join(gitRoot, ".git", "phantom", "gardens");
    const gardenPath = join(gardensPath, name);

    // Check if garden exists
    try {
      await access(gardenPath);
    } catch {
      return {
        success: false,
        message: `Error: Garden '${name}' does not exist`,
      };
    }

    // Check for uncommitted changes
    let hasUncommittedChanges = false;
    let changedFiles = 0;
    try {
      const { stdout } = await execAsync("git status --porcelain", {
        cwd: gardenPath,
      });
      const changes = stdout.trim();
      if (changes) {
        hasUncommittedChanges = true;
        changedFiles = changes.split("\n").length;
      }
    } catch {
      // If git status fails, assume no changes
      hasUncommittedChanges = false;
    }

    // If garden has uncommitted changes and --force is not specified, refuse deletion
    if (hasUncommittedChanges && !force) {
      return {
        success: false,
        message: `Error: Garden '${name}' has uncommitted changes (${changedFiles} files). Use --force to delete anyway.`,
        hasUncommittedChanges: true,
        changedFiles,
      };
    }

    // Remove git worktree
    try {
      await execAsync(`git worktree remove "${gardenPath}"`, {
        cwd: gitRoot,
      });
    } catch (error) {
      // If worktree remove fails, try force removal
      try {
        await execAsync(`git worktree remove --force "${gardenPath}"`, {
          cwd: gitRoot,
        });
      } catch {
        return {
          success: false,
          message: `Error: Failed to remove worktree for garden '${name}'`,
        };
      }
    }

    // Delete associated branch
    const branchName = `phantom/gardens/${name}`;
    try {
      await execAsync(`git branch -D "${branchName}"`, {
        cwd: gitRoot,
      });
    } catch {
      // Branch might not exist or already deleted - this is not an error
      // We'll still report success for the worktree removal
    }

    let message = `Deleted garden '${name}' and its branch '${branchName}'`;
    if (hasUncommittedChanges) {
      message = `Warning: Garden '${name}' had uncommitted changes (${changedFiles} files)\n${message}`;
    }

    return {
      success: true,
      message,
      hasUncommittedChanges,
      changedFiles: hasUncommittedChanges ? changedFiles : undefined,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return {
      success: false,
      message: `Error deleting garden: ${errorMessage}`,
    };
  }
}

export async function gardensDeleteHandler(args: string[]): Promise<void> {
  // Parse arguments for --force flag
  const forceIndex = args.indexOf("--force");
  const force = forceIndex !== -1;

  // Remove --force from args to get the garden name
  const filteredArgs = args.filter((arg) => arg !== "--force");
  const name = filteredArgs[0];

  const result = await deleteGarden(name, { force });

  if (!result.success) {
    console.error(result.message);
    exit(1);
  }

  console.log(result.message);
}
