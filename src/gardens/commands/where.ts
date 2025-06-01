import { access } from "node:fs/promises";
import { join } from "node:path";
import { exit } from "node:process";
import { getGitRoot } from "../../git/libs/get-git-root.ts";

export async function whereGarden(name: string): Promise<{
  success: boolean;
  message?: string;
  path?: string;
}> {
  if (!name) {
    return { success: false, message: "Error: garden name required" };
  }

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

    return {
      success: true,
      path: gardenPath,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return {
      success: false,
      message: `Error locating garden: ${errorMessage}`,
    };
  }
}

export async function gardensWhereHandler(args: string[]): Promise<void> {
  const name = args[0];
  const result = await whereGarden(name);

  if (!result.success) {
    console.error(result.message);
    exit(1);
  }

  console.log(result.path);
}
