import { deepStrictEqual, strictEqual } from "node:assert";
import { before, describe, it, mock } from "node:test";

describe("listGardens", () => {
  let accessMock: ReturnType<typeof mock.fn>;
  let readdirMock: ReturnType<typeof mock.fn>;
  let execMock: ReturnType<typeof mock.fn>;
  let listGardens: typeof import("./list.ts").listGardens;

  before(async () => {
    accessMock = mock.fn();
    readdirMock = mock.fn();
    execMock = mock.fn();

    mock.module("node:fs/promises", {
      namedExports: {
        access: accessMock,
        readdir: readdirMock,
      },
    });

    mock.module("node:child_process", {
      namedExports: {
        exec: execMock,
      },
    });

    mock.module("node:util", {
      namedExports: {
        promisify: (fn: unknown) => fn,
      },
    });

    ({ listGardens } = await import("./list.ts"));
  });

  it("should return empty array when gardens directory doesn't exist", async () => {
    accessMock.mock.resetCalls();
    readdirMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock gardens directory doesn't exist
    accessMock.mock.mockImplementation((path: string) => {
      if (path === "/test/repo/.git/phantom/gardens") {
        return Promise.reject(new Error("ENOENT"));
      }
      return Promise.resolve();
    });

    const result = await listGardens();

    strictEqual(result.success, true);
    deepStrictEqual(result.gardens, []);
    strictEqual(
      result.message,
      "No gardens found (gardens directory doesn't exist)",
    );
  });

  it("should return empty array when gardens directory is empty", async () => {
    accessMock.mock.resetCalls();
    readdirMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock gardens directory exists but is empty
    accessMock.mock.mockImplementation(() => Promise.resolve());
    readdirMock.mock.mockImplementation(() => Promise.resolve([]));

    const result = await listGardens();

    strictEqual(result.success, true);
    deepStrictEqual(result.gardens, []);
    strictEqual(result.message, "No gardens found");
  });

  it("should list gardens with clean status", async () => {
    accessMock.mock.resetCalls();
    readdirMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot and git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git branch --show-current") {
          if (options?.cwd?.includes("test-garden-1")) {
            return Promise.resolve({ stdout: "feature/test\n", stderr: "" });
          }
          if (options?.cwd?.includes("test-garden-2")) {
            return Promise.resolve({ stdout: "main\n", stderr: "" });
          }
        }
        if (cmd === "git status --porcelain") {
          return Promise.resolve({ stdout: "", stderr: "" }); // Clean status
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock gardens directory and contents
    accessMock.mock.mockImplementation(() => Promise.resolve());
    readdirMock.mock.mockImplementation(() =>
      Promise.resolve(["test-garden-1", "test-garden-2"]),
    );

    const result = await listGardens();

    strictEqual(result.success, true);
    strictEqual(result.gardens?.length, 2);
    strictEqual(result.gardens?.[0].name, "test-garden-1");
    strictEqual(result.gardens?.[0].branch, "feature/test");
    strictEqual(result.gardens?.[0].status, "clean");
    strictEqual(result.gardens?.[1].name, "test-garden-2");
    strictEqual(result.gardens?.[1].branch, "main");
    strictEqual(result.gardens?.[1].status, "clean");
  });

  it("should list gardens with dirty status", async () => {
    accessMock.mock.resetCalls();
    readdirMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot and git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git branch --show-current") {
          return Promise.resolve({ stdout: "feature/dirty\n", stderr: "" });
        }
        if (cmd === "git status --porcelain") {
          return Promise.resolve({
            stdout: " M file1.ts\n?? file2.ts\n",
            stderr: "",
          }); // Dirty status with 2 files
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock gardens directory and contents
    accessMock.mock.mockImplementation(() => Promise.resolve());
    readdirMock.mock.mockImplementation(() => Promise.resolve(["dirty-garden"]));

    const result = await listGardens();

    strictEqual(result.success, true);
    strictEqual(result.gardens?.length, 1);
    strictEqual(result.gardens?.[0].name, "dirty-garden");
    strictEqual(result.gardens?.[0].branch, "feature/dirty");
    strictEqual(result.gardens?.[0].status, "dirty");
    strictEqual(result.gardens?.[0].changedFiles, 2);
  });

  it("should handle git command errors gracefully", async () => {
    accessMock.mock.resetCalls();
    readdirMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot and failing git commands
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
      }
      if (cmd === "git branch --show-current") {
        return Promise.reject(new Error("Not a git repository"));
      }
      if (cmd === "git status --porcelain") {
        return Promise.reject(new Error("Git command failed"));
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock gardens directory and contents
    accessMock.mock.mockImplementation(() => Promise.resolve());
    readdirMock.mock.mockImplementation(() => Promise.resolve(["error-garden"]));

    const result = await listGardens();

    strictEqual(result.success, true);
    strictEqual(result.gardens?.length, 1);
    strictEqual(result.gardens?.[0].name, "error-garden");
    strictEqual(result.gardens?.[0].branch, "unknown");
    strictEqual(result.gardens?.[0].status, "clean");
  });

  it("should handle detached HEAD state", async () => {
    accessMock.mock.resetCalls();
    readdirMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot and git commands
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
      }
      if (cmd === "git branch --show-current") {
        return Promise.resolve({ stdout: "\n", stderr: "" }); // Empty output = detached HEAD
      }
      if (cmd === "git status --porcelain") {
        return Promise.resolve({ stdout: "", stderr: "" });
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock gardens directory and contents
    accessMock.mock.mockImplementation(() => Promise.resolve());
    readdirMock.mock.mockImplementation(() =>
      Promise.resolve(["detached-garden"]),
    );

    const result = await listGardens();

    strictEqual(result.success, true);
    strictEqual(result.gardens?.length, 1);
    strictEqual(result.gardens?.[0].name, "detached-garden");
    strictEqual(result.gardens?.[0].branch, "detached HEAD");
    strictEqual(result.gardens?.[0].status, "clean");
  });
});
