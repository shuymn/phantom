import { parseArgs } from "node:util";
import { getGitRoot } from "../../core/git/libs/get-git-root.ts";
import { getPhantomEnv } from "../../core/process/env.ts";
import {
  executeKittyCommand,
  isInsideKitty,
} from "../../core/process/kitty.ts";
import { shellInWorktree as shellInWorktreeCore } from "../../core/process/shell.ts";
import { executeTmuxCommand, isInsideTmux } from "../../core/process/tmux.ts";
import { isErr } from "../../core/types/result.ts";
import { WorktreeNotFoundError } from "../../core/worktree/errors.ts";
import { selectWorktreeWithFzf } from "../../core/worktree/select.ts";
import { validateWorktreeExists } from "../../core/worktree/validate.ts";
import { exitCodes, exitWithError, exitWithSuccess } from "../errors.ts";
import { output } from "../output.ts";

export async function shellHandler(args: string[]): Promise<void> {
  const { positionals, values } = parseArgs({
    args,
    options: {
      fzf: {
        type: "boolean",
        default: false,
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
    },
    strict: true,
    allowPositionals: true,
  });

  const useFzf = values.fzf ?? false;

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

  // Check for mutually exclusive options
  if (tmuxOption && kittyOption) {
    exitWithError(
      "Cannot use both tmux and kitty options simultaneously",
      exitCodes.validationError,
    );
  }

  if (positionals.length === 0 && !useFzf) {
    exitWithError(
      "Usage: phantom shell <worktree-name> or phantom shell --fzf",
      exitCodes.validationError,
    );
  }

  if (positionals.length > 0 && useFzf) {
    exitWithError(
      "Cannot specify both a worktree name and --fzf option",
      exitCodes.validationError,
    );
  }

  let worktreeName: string;

  try {
    const gitRoot = await getGitRoot();

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

    if (useFzf) {
      const selectResult = await selectWorktreeWithFzf(gitRoot);
      if (isErr(selectResult)) {
        exitWithError(selectResult.error.message, exitCodes.generalError);
      }
      if (!selectResult.value) {
        exitWithSuccess();
      }
      worktreeName = selectResult.value.name;
    } else {
      worktreeName = positionals[0];
    }

    // Get worktree path for display
    const validation = await validateWorktreeExists(gitRoot, worktreeName);
    if (isErr(validation)) {
      exitWithError(validation.error.message, exitCodes.generalError);
    }

    if (tmuxDirection) {
      output.log(
        `Opening worktree '${worktreeName}' in tmux ${
          tmuxDirection === "new" ? "window" : "pane"
        }...`,
      );

      const shell = process.env.SHELL || "/bin/sh";

      const tmuxResult = await executeTmuxCommand({
        direction: tmuxDirection,
        command: shell,
        cwd: validation.value.path,
        env: getPhantomEnv(worktreeName, validation.value.path),
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

      exitWithSuccess();
    }

    if (kittyDirection) {
      output.log(
        `Opening worktree '${worktreeName}' in kitty ${
          kittyDirection === "new" ? "tab" : "split"
        }...`,
      );

      const shell = process.env.SHELL || "/bin/sh";

      const kittyResult = await executeKittyCommand({
        direction: kittyDirection,
        command: shell,
        cwd: validation.value.path,
        env: getPhantomEnv(worktreeName, validation.value.path),
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

      exitWithSuccess();
    }

    output.log(
      `Entering worktree '${worktreeName}' at ${validation.value.path}`,
    );
    output.log("Type 'exit' to return to your original directory\n");

    const result = await shellInWorktreeCore(gitRoot, worktreeName);

    if (isErr(result)) {
      const exitCode =
        result.error instanceof WorktreeNotFoundError
          ? exitCodes.notFound
          : result.error.exitCode || exitCodes.generalError;
      exitWithError(result.error.message, exitCode);
    }

    process.exit(result.value.exitCode);
  } catch (error) {
    exitWithError(
      error instanceof Error ? error.message : String(error),
      exitCodes.generalError,
    );
  }
}
