import { access, mkdir } from "node:fs/promises";
import { join } from "node:path";
import { exit } from "node:process";
import { addWorktree } from "../../git/libs/add-worktree.ts";
import { getGitRoot } from "../../git/libs/get-git-root.ts";

export async function createGarden(name: string): Promise<{
  success: boolean;
  message: string;
  path?: string;
}> {
  if (!name) {
    return { success: false, message: "Error: garden name required" };
  }

  try {
    const gitRoot = await getGitRoot();
    const gardensPath = join(gitRoot, ".git", "phantom", "gardens");
    const worktreePath = join(gardensPath, name);

    try {
      await access(gardensPath);
    } catch {
      await mkdir(gardensPath, { recursive: true });
    }

    try {
      await access(worktreePath);
      return {
        success: false,
        message: `Error: garden '${name}' already exists`,
      };
    } catch {
      // Path doesn't exist, which is what we want
    }

    await addWorktree({
      path: worktreePath,
      branch: `phantom/gardens/${name}`,
      commitish: "HEAD",
    });

    return {
      success: true,
      message: `Created garden '${name}' at ${worktreePath}`,
      path: worktreePath,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return { success: false, message: `Error creating garden: ${errorMessage}` };
  }
}

export async function gardensCreateHandler(args: string[]): Promise<void> {
  const name = args[0];
  const result = await createGarden(name);

  if (!result.success) {
    console.error(result.message);
    exit(1);
  }

  console.log(result.message);
}
