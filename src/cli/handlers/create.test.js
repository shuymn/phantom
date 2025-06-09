import { rejects, strictEqual } from "node:assert";
import { describe, it, mock } from "node:test";
import { err, ok } from "../../core/types/result.ts";

const exitMock = mock.fn();
const consoleLogMock = mock.fn();
const consoleErrorMock = mock.fn();
const getGitRootMock = mock.fn();
const createWorktreeMock = mock.fn();
const execInWorktreeMock = mock.fn();
const shellInWorktreeMock = mock.fn();
const isInsideTmuxMock = mock.fn();
const executeTmuxCommandMock = mock.fn();
const isInsideKittyMock = mock.fn();
const executeKittyCommandMock = mock.fn();
const exitWithErrorMock = mock.fn((message, code) => {
  if (message) consoleErrorMock(`Error: ${message}`);
  exitMock(code);
  throw new Error(`Exit with code ${code}: ${message}`);
});
const exitWithSuccessMock = mock.fn(() => {
  exitMock(0);
  throw new Error("Exit with code 0");
});

// Mock process module
const processEnvMock = {};
mock.module("node:process", {
  namedExports: {
    exit: exitMock,
    env: processEnvMock,
  },
});

mock.module("../../core/git/libs/get-git-root.ts", {
  namedExports: {
    getGitRoot: getGitRootMock,
  },
});

mock.module("../../core/worktree/create.ts", {
  namedExports: {
    createWorktree: createWorktreeMock,
  },
});

mock.module("../../core/process/exec.ts", {
  namedExports: {
    execInWorktree: execInWorktreeMock,
  },
});

mock.module("../../core/process/shell.ts", {
  namedExports: {
    shellInWorktree: shellInWorktreeMock,
  },
});

mock.module("../../core/process/tmux.ts", {
  namedExports: {
    isInsideTmux: isInsideTmuxMock,
    executeTmuxCommand: executeTmuxCommandMock,
  },
});

mock.module("../../core/process/kitty.ts", {
  namedExports: {
    isInsideKitty: isInsideKittyMock,
    executeKittyCommand: executeKittyCommandMock,
  },
});

mock.module("../output.ts", {
  namedExports: {
    output: {
      log: consoleLogMock,
      error: consoleErrorMock,
    },
  },
});

mock.module("../errors.ts", {
  namedExports: {
    exitCodes: {
      generalError: 1,
      validationError: 2,
    },
    exitWithError: exitWithErrorMock,
    exitWithSuccess: exitWithSuccessMock,
  },
});

const { createHandler } = await import("./create.ts");

describe("createHandler", () => {
  const resetMocks = () => {
    // Reset all mocks
    exitMock.mock.resetCalls();
    consoleLogMock.mock.resetCalls();
    consoleErrorMock.mock.resetCalls();
    getGitRootMock.mock.resetCalls();
    createWorktreeMock.mock.resetCalls();
    execInWorktreeMock.mock.resetCalls();
    shellInWorktreeMock.mock.resetCalls();
    isInsideTmuxMock.mock.resetCalls();
    executeTmuxCommandMock.mock.resetCalls();
    isInsideKittyMock.mock.resetCalls();
    executeKittyCommandMock.mock.resetCalls();
    exitWithErrorMock.mock.resetCalls();
    exitWithSuccessMock.mock.resetCalls();

    // Clear process env
    for (const key in processEnvMock) {
      delete processEnvMock[key];
    }
  };

  it("should create worktree and execute command with --exec option", async () => {
    resetMocks();
    processEnvMock.SHELL = "/bin/bash";
    getGitRootMock.mock.mockImplementation(() => Promise.resolve("/test/repo"));
    createWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(
        ok({
          message:
            "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
          path: "/test/repo/.git/phantom/worktrees/feature",
        }),
      ),
    );
    execInWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(ok({ exitCode: 0 })),
    );

    await rejects(
      async () => await createHandler(["feature", "--exec", "echo hello"]),
      /Exit with code 0/,
    );

    strictEqual(createWorktreeMock.mock.calls.length, 1);
    strictEqual(createWorktreeMock.mock.calls[0].arguments[0], "/test/repo");
    strictEqual(createWorktreeMock.mock.calls[0].arguments[1], "feature");

    strictEqual(execInWorktreeMock.mock.calls.length, 1);
    strictEqual(execInWorktreeMock.mock.calls[0].arguments[0], "/test/repo");
    strictEqual(execInWorktreeMock.mock.calls[0].arguments[1], "feature");
    const execArgs = execInWorktreeMock.mock.calls[0].arguments[2];
    strictEqual(execArgs[0], "/bin/bash");
    strictEqual(execArgs[1], "-c");
    strictEqual(execArgs[2], "echo hello");

    strictEqual(consoleLogMock.mock.calls.length, 2);
    strictEqual(
      consoleLogMock.mock.calls[0].arguments[0],
      "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
    );
    strictEqual(
      consoleLogMock.mock.calls[1].arguments[0],
      "\nExecuting command in worktree 'feature': echo hello",
    );

    strictEqual(exitMock.mock.calls[0].arguments[0], 0);
  });

  it("should handle exec command failure", async () => {
    resetMocks();
    processEnvMock.SHELL = "/bin/bash";
    getGitRootMock.mock.mockImplementation(() => Promise.resolve("/test/repo"));
    createWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(
        ok({
          message:
            "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
          path: "/test/repo/.git/phantom/worktrees/feature",
        }),
      ),
    );
    execInWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(
        err({
          message: "Command failed",
          exitCode: 1,
        }),
      ),
    );

    await rejects(
      async () => await createHandler(["feature", "--exec", "false"]),
      /Exit with code 1/,
    );

    strictEqual(createWorktreeMock.mock.calls.length, 1);
    strictEqual(execInWorktreeMock.mock.calls.length, 1);
    strictEqual(
      consoleErrorMock.mock.calls[0].arguments[0],
      "Error: Command failed",
    );
    strictEqual(exitMock.mock.calls[0].arguments[0], 1);
  });

  it("should error when --shell and --exec are used together", async () => {
    resetMocks();
    await rejects(
      async () =>
        await createHandler(["feature", "--shell", "--exec", "echo hello"]),
      /Exit with code 2/,
    );

    strictEqual(consoleErrorMock.mock.calls.length, 1);
    strictEqual(
      consoleErrorMock.mock.calls[0].arguments[0],
      "Error: Cannot use --shell and --exec together",
    );
    strictEqual(exitMock.mock.calls[0].arguments[0], 2);
  });

  it("should use /bin/sh when SHELL env var is not set", async () => {
    resetMocks();
    // No SHELL env var set
    getGitRootMock.mock.mockImplementation(() => Promise.resolve("/test/repo"));
    createWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(
        ok({
          message:
            "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
          path: "/test/repo/.git/phantom/worktrees/feature",
        }),
      ),
    );
    execInWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(ok({ exitCode: 0 })),
    );

    await rejects(
      async () => await createHandler(["feature", "--exec", "echo hello"]),
      /Exit with code 0/,
    );

    const execArgs = execInWorktreeMock.mock.calls[0].arguments[2];
    strictEqual(execArgs[0], "/bin/sh");
  });

  it("should error when --tmux is used outside tmux session", async () => {
    resetMocks();
    isInsideTmuxMock.mock.mockImplementation(() => Promise.resolve(false));

    await rejects(
      async () => await createHandler(["feature", "--tmux"]),
      /Exit with code 2/,
    );

    strictEqual(consoleErrorMock.mock.calls.length, 1);
    strictEqual(
      consoleErrorMock.mock.calls[0].arguments[0],
      "Error: The --tmux option can only be used inside a tmux session",
    );
    strictEqual(exitMock.mock.calls[0].arguments[0], 2);
  });

  it("should create worktree and open in tmux window", async () => {
    resetMocks();
    processEnvMock.SHELL = "/bin/bash";
    processEnvMock.TMUX = "/tmp/tmux-1000/default,12345,0";
    getGitRootMock.mock.mockImplementation(() => Promise.resolve("/test/repo"));
    createWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(
        ok({
          message:
            "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
          path: "/test/repo/.git/phantom/worktrees/feature",
        }),
      ),
    );
    isInsideTmuxMock.mock.mockImplementation(() => Promise.resolve(true));
    executeTmuxCommandMock.mock.mockImplementation(() =>
      Promise.resolve(ok({ exitCode: 0 })),
    );

    await rejects(
      async () => await createHandler(["feature", "--tmux"]),
      /Exit with code 0/,
    );

    strictEqual(createWorktreeMock.mock.calls.length, 1);
    strictEqual(executeTmuxCommandMock.mock.calls.length, 1);

    // Verify tmux command was called with correct arguments
    const tmuxArgs = executeTmuxCommandMock.mock.calls[0].arguments[0];
    strictEqual(tmuxArgs.direction, "new");
    strictEqual(tmuxArgs.cwd, "/test/repo/.git/phantom/worktrees/feature");
    strictEqual(tmuxArgs.windowName, "feature");

    strictEqual(
      consoleLogMock.mock.calls[0].arguments[0],
      "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
    );
    strictEqual(
      consoleLogMock.mock.calls[1].arguments[0],
      "\nOpening worktree 'feature' in tmux window...",
    );

    strictEqual(exitMock.mock.calls[0].arguments[0], 0);
  });

  it("should create worktree and open in tmux pane with vertical split", async () => {
    resetMocks();
    processEnvMock.SHELL = "/bin/bash";
    processEnvMock.TMUX = "/tmp/tmux-1000/default,12345,0";
    getGitRootMock.mock.mockImplementation(() => Promise.resolve("/test/repo"));
    createWorktreeMock.mock.mockImplementation(() =>
      Promise.resolve(
        ok({
          message:
            "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
          path: "/test/repo/.git/phantom/worktrees/feature",
        }),
      ),
    );
    isInsideTmuxMock.mock.mockImplementation(() => Promise.resolve(true));
    executeTmuxCommandMock.mock.mockImplementation(() =>
      Promise.resolve(ok({ exitCode: 0 })),
    );

    await rejects(
      async () => await createHandler(["feature", "--tmux-vertical"]),
      /Exit with code 0/,
    );

    strictEqual(createWorktreeMock.mock.calls.length, 1);
    strictEqual(executeTmuxCommandMock.mock.calls.length, 1);

    // Verify tmux command was called with correct arguments
    const tmuxArgs = executeTmuxCommandMock.mock.calls[0].arguments[0];
    strictEqual(tmuxArgs.direction, "vertical");
    strictEqual(tmuxArgs.cwd, "/test/repo/.git/phantom/worktrees/feature");
    strictEqual(tmuxArgs.windowName, undefined);

    strictEqual(
      consoleLogMock.mock.calls[0].arguments[0],
      "Created worktree 'feature' at /test/repo/.git/phantom/worktrees/feature",
    );
    strictEqual(
      consoleLogMock.mock.calls[1].arguments[0],
      "\nOpening worktree 'feature' in tmux pane...",
    );

    strictEqual(exitMock.mock.calls[0].arguments[0], 0);
  });

  it("should error when multiple action options are used together", async () => {
    resetMocks();
    await rejects(
      async () => await createHandler(["feature", "--shell", "--tmux"]),
      /Exit with code 2/,
    );

    strictEqual(consoleErrorMock.mock.calls.length, 1);
    strictEqual(
      consoleErrorMock.mock.calls[0].arguments[0],
      "Error: Cannot use --shell, --exec, --tmux, and --kitty options together",
    );
    strictEqual(exitMock.mock.calls[0].arguments[0], 2);
  });

  it("should error when kitty option used outside kitty", async () => {
    resetMocks();

    getGitRootMock.mock.mockImplementation(() => "/repo");
    isInsideKittyMock.mock.mockImplementation(() => false);

    await rejects(
      async () => await createHandler(["feature", "--kitty"]),
      /Exit with code 2: The --kitty option can only be used inside a kitty terminal/,
    );

    strictEqual(isInsideKittyMock.mock.calls.length, 1);
  });

  it("should create worktree and open in new kitty tab", async () => {
    resetMocks();

    getGitRootMock.mock.mockImplementation(() => "/repo");
    isInsideKittyMock.mock.mockImplementation(() => true);
    createWorktreeMock.mock.mockImplementation(() =>
      ok({ path: "/repo/.git/phantom/worktrees/feature" }),
    );
    executeKittyCommandMock.mock.mockImplementation(() => ok({ exitCode: 0 }));

    await rejects(
      async () => await createHandler(["feature", "--kitty"]),
      /Exit with code 0/,
    );

    strictEqual(isInsideKittyMock.mock.calls.length, 1);
    strictEqual(createWorktreeMock.mock.calls.length, 1);
    strictEqual(executeKittyCommandMock.mock.calls.length, 1);

    const kittyCall = executeKittyCommandMock.mock.calls[0].arguments[0];
    strictEqual(kittyCall.direction, "new");
    strictEqual(kittyCall.cwd, "/repo/.git/phantom/worktrees/feature");
    strictEqual(kittyCall.windowTitle, "feature");
  });

  it("should create worktree and open in vertical kitty split", async () => {
    resetMocks();

    getGitRootMock.mock.mockImplementation(() => "/repo");
    isInsideKittyMock.mock.mockImplementation(() => true);
    createWorktreeMock.mock.mockImplementation(() =>
      ok({ path: "/repo/.git/phantom/worktrees/feature" }),
    );
    executeKittyCommandMock.mock.mockImplementation(() => ok({ exitCode: 0 }));

    await rejects(
      async () => await createHandler(["feature", "--kitty-v"]),
      /Exit with code 0/,
    );

    const kittyCall = executeKittyCommandMock.mock.calls[0].arguments[0];
    strictEqual(kittyCall.direction, "vertical");
    strictEqual(kittyCall.windowTitle, undefined);
  });

  it("should error when both tmux and kitty options are used", async () => {
    resetMocks();

    await rejects(
      async () => await createHandler(["feature", "--tmux", "--kitty"]),
      /Exit with code 2: Cannot use --shell, --exec, --tmux, and --kitty options together/,
    );

    strictEqual(consoleErrorMock.mock.calls.length, 1);
  });
});
