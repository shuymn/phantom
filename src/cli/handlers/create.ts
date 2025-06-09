import { parseArgs } from "node:util";
import {
  ConfigNotFoundError,
  ConfigParseError,
  loadConfig,
} from "../../core/config/loader.ts";
import { ConfigValidationError } from "../../core/config/validate.ts";
import { getGitRoot } from "../../core/git/libs/get-git-root.ts";
import { getPhantomEnv } from "../../core/process/env.ts";
import { execInWorktree } from "../../core/process/exec.ts";
import {
  executeKittyCommand,
  isInsideKitty,
} from "../../core/process/kitty.ts";
import { shellInWorktree } from "../../core/process/shell.ts";
import { executeTmuxCommand, isInsideTmux } from "../../core/process/tmux.ts";
import { isErr, isOk } from "../../core/types/result.ts";
import { createWorktree as createWorktreeCore } from "../../core/worktree/create.ts";
import { WorktreeAlreadyExistsError } from "../../core/worktree/errors.ts";
import { exitCodes, exitWithError, exitWithSuccess } from "../errors.ts";
import { output } from "../output.ts";

export async function createHandler(args: string[]): Promise<void> {
  const { values, positionals } = parseArgs({
    args,
    options: {
      shell: {
        type: "boolean",
        short: "s",
      },
      exec: {
        type: "string",
        short: "x",
      },
      tmux: {
        type: "boolean",
        short: "t",
      },
      "tmux-vertical": {
        type: "boolean",
      },
      "tmux-v": {
        type: "boolean",
      },
      "tmux-horizontal": {
        type: "boolean",
      },
      "tmux-h": {
        type: "boolean",
      },
      kitty: {
        type: "boolean",
        short: "k",
      },
      "kitty-vertical": {
        type: "boolean",
      },
      "kitty-v": {
        type: "boolean",
      },
      "kitty-horizontal": {
        type: "boolean",
      },
      "kitty-h": {
        type: "boolean",
      },
      "copy-file": {
        type: "string",
        multiple: true,
      },
    },
    strict: true,
    allowPositionals: true,
  });

  if (positionals.length === 0) {
    exitWithError(
      "Please provide a name for the new worktree",
      exitCodes.validationError,
    );
  }

  const worktreeName = positionals[0];
  const openShell = values.shell ?? false;
  const execCommand = values.exec;
  const copyFileOptions = values["copy-file"];

  // Determine tmux option
  const tmuxOption =
    values.tmux ||
    values["tmux-vertical"] ||
    values["tmux-v"] ||
    values["tmux-horizontal"] ||
    values["tmux-h"];

  let tmuxDirection: "new" | "vertical" | "horizontal" | undefined;
  if (values.tmux) {
    tmuxDirection = "new";
  } else if (values["tmux-vertical"] || values["tmux-v"]) {
    tmuxDirection = "vertical";
  } else if (values["tmux-horizontal"] || values["tmux-h"]) {
    tmuxDirection = "horizontal";
  }

  // Determine kitty option
  const kittyOption =
    values.kitty ||
    values["kitty-vertical"] ||
    values["kitty-v"] ||
    values["kitty-horizontal"] ||
    values["kitty-h"];

  let kittyDirection: "new" | "vertical" | "horizontal" | undefined;
  if (values.kitty) {
    kittyDirection = "new";
  } else if (values["kitty-vertical"] || values["kitty-v"]) {
    kittyDirection = "vertical";
  } else if (values["kitty-horizontal"] || values["kitty-h"]) {
    kittyDirection = "horizontal";
  }

  if (
    [openShell, execCommand !== undefined, tmuxOption, kittyOption].filter(
      Boolean,
    ).length > 1
  ) {
    exitWithError(
      "Cannot use --shell, --exec, --tmux, and --kitty options together",
      exitCodes.validationError,
    );
  }

  if (tmuxOption && !(await isInsideTmux())) {
    exitWithError(
      "The --tmux option can only be used inside a tmux session",
      exitCodes.validationError,
    );
  }

  if (kittyOption && !(await isInsideKitty())) {
    exitWithError(
      "The --kitty option can only be used inside a kitty terminal",
      exitCodes.validationError,
    );
  }

  try {
    const gitRoot = await getGitRoot();

    let filesToCopy: string[] = [];

    // Load files from config
    const configResult = await loadConfig(gitRoot);
    if (isOk(configResult)) {
      if (configResult.value.postCreate?.copyFiles) {
        filesToCopy = [...configResult.value.postCreate.copyFiles];
      }
    } else {
      // Display warning for validation and parse errors
      if (configResult.error instanceof ConfigValidationError) {
        output.warn(`Configuration warning: ${configResult.error.message}`);
      } else if (configResult.error instanceof ConfigParseError) {
        output.warn(`Configuration warning: ${configResult.error.message}`);
      }
      // ConfigNotFoundError remains silent as the config file is optional
    }

    // Add files from CLI options
    if (copyFileOptions && copyFileOptions.length > 0) {
      const cliFiles = Array.isArray(copyFileOptions)
        ? copyFileOptions
        : [copyFileOptions];
      // Merge with config files, removing duplicates
      filesToCopy = [...new Set([...filesToCopy, ...cliFiles])];
    }

    const result = await createWorktreeCore(gitRoot, worktreeName, {
      copyFiles: filesToCopy.length > 0 ? filesToCopy : undefined,
    });

    if (isErr(result)) {
      const exitCode =
        result.error instanceof WorktreeAlreadyExistsError
          ? exitCodes.validationError
          : exitCodes.generalError;
      exitWithError(result.error.message, exitCode);
    }

    output.log(result.value.message);

    if (result.value.copyError) {
      output.error(
        `\nWarning: Failed to copy some files: ${result.value.copyError}`,
      );
    }

    // Execute post-create commands from config
    if (isOk(configResult) && configResult.value.postCreate?.commands) {
      const commands = configResult.value.postCreate.commands;
      output.log("\nRunning post-create commands...");

      for (const command of commands) {
        output.log(`Executing: ${command}`);
        const shell = process.env.SHELL || "/bin/sh";
        const cmdResult = await execInWorktree(gitRoot, worktreeName, [
          shell,
          "-c",
          command,
        ]);

        if (isErr(cmdResult)) {
          output.error(`Failed to execute command: ${cmdResult.error.message}`);
          const exitCode =
            "exitCode" in cmdResult.error
              ? (cmdResult.error.exitCode ?? exitCodes.generalError)
              : exitCodes.generalError;
          exitWithError(`Post-create command failed: ${command}`, exitCode);
        }

        // Check exit code
        if (cmdResult.value.exitCode !== 0) {
          exitWithError(
            `Post-create command failed: ${command}`,
            cmdResult.value.exitCode,
          );
        }
      }
    }

    if (execCommand && isOk(result)) {
      output.log(
        `\nExecuting command in worktree '${worktreeName}': ${execCommand}`,
      );

      const shell = process.env.SHELL || "/bin/sh";
      const execResult = await execInWorktree(
        gitRoot,
        worktreeName,
        [shell, "-c", execCommand],
        { interactive: true },
      );

      if (isErr(execResult)) {
        output.error(execResult.error.message);
        const exitCode =
          "exitCode" in execResult.error
            ? (execResult.error.exitCode ?? exitCodes.generalError)
            : exitCodes.generalError;
        exitWithError("", exitCode);
      }

      process.exit(execResult.value.exitCode ?? 0);
    }

    if (openShell && isOk(result)) {
      output.log(
        `\nEntering worktree '${worktreeName}' at ${result.value.path}`,
      );
      output.log("Type 'exit' to return to your original directory\n");

      const shellResult = await shellInWorktree(gitRoot, worktreeName);

      if (isErr(shellResult)) {
        output.error(shellResult.error.message);
        const exitCode =
          "exitCode" in shellResult.error
            ? (shellResult.error.exitCode ?? exitCodes.generalError)
            : exitCodes.generalError;
        exitWithError("", exitCode);
      }

      process.exit(shellResult.value.exitCode ?? 0);
    }

    if (tmuxDirection && isOk(result)) {
      output.log(
        `\nOpening worktree '${worktreeName}' in tmux ${
          tmuxDirection === "new" ? "window" : "pane"
        }...`,
      );

      const shell = process.env.SHELL || "/bin/sh";

      const tmuxResult = await executeTmuxCommand({
        direction: tmuxDirection,
        command: shell,
        cwd: result.value.path,
        env: getPhantomEnv(worktreeName, result.value.path),
        windowName: tmuxDirection === "new" ? worktreeName : undefined,
      });

      if (isErr(tmuxResult)) {
        output.error(tmuxResult.error.message);
        const exitCode =
          "exitCode" in tmuxResult.error
            ? (tmuxResult.error.exitCode ?? exitCodes.generalError)
            : exitCodes.generalError;
        exitWithError("", exitCode);
      }
    }

    if (kittyDirection && isOk(result)) {
      output.log(
        `\nOpening worktree '${worktreeName}' in kitty ${
          kittyDirection === "new" ? "tab" : "split"
        }...`,
      );

      const shell = process.env.SHELL || "/bin/sh";

      const kittyResult = await executeKittyCommand({
        direction: kittyDirection,
        command: shell,
        cwd: result.value.path,
        env: getPhantomEnv(worktreeName, result.value.path),
        windowTitle: kittyDirection === "new" ? worktreeName : undefined,
      });

      if (isErr(kittyResult)) {
        output.error(kittyResult.error.message);
        const exitCode =
          "exitCode" in kittyResult.error
            ? (kittyResult.error.exitCode ?? exitCodes.generalError)
            : exitCodes.generalError;
        exitWithError("", exitCode);
      }
    }

    exitWithSuccess();
  } catch (error) {
    exitWithError(
      error instanceof Error ? error.message : String(error),
      exitCodes.generalError,
    );
  }
}
