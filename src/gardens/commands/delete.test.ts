import { strictEqual } from "node:assert";
import { before, describe, it, mock } from "node:test";

describe("deleteGarden", () => {
  let accessMock: ReturnType<typeof mock.fn>;
  let execMock: ReturnType<typeof mock.fn>;
  let deleteGarden: typeof import("./delete.ts").deleteGarden;

  before(async () => {
    accessMock = mock.fn();
    execMock = mock.fn();

    mock.module("node:fs/promises", {
      namedExports: {
        access: accessMock,
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

    ({ deleteGarden } = await import("./delete.ts"));
  });

  it("should return error when name is not provided", async () => {
    const result = await deleteGarden("");
    strictEqual(result.success, false);
    strictEqual(result.message, "Error: garden name required");
  });

  it("should return error when garden does not exist", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock garden doesn't exist
    accessMock.mock.mockImplementation(() => {
      return Promise.reject(new Error("ENOENT"));
    });

    const result = await deleteGarden("nonexistent-garden");

    strictEqual(result.success, false);
    strictEqual(
      result.message,
      "Error: Garden 'nonexistent-garden' does not exist",
    );
  });

  it("should delete clean garden successfully", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git status --porcelain") {
          return Promise.resolve({ stdout: "", stderr: "" }); // Clean status
        }
        if (cmd.includes("git worktree remove")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git branch -D")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await deleteGarden("clean-garden");

    strictEqual(result.success, true);
    strictEqual(
      result.message,
      "Deleted garden 'clean-garden' and its branch 'phantom/gardens/clean-garden'",
    );
    strictEqual(result.hasUncommittedChanges, false);
  });

  it("should refuse to delete dirty garden without --force", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
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

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await deleteGarden("dirty-garden");

    strictEqual(result.success, false);
    strictEqual(
      result.message,
      "Error: Garden 'dirty-garden' has uncommitted changes (2 files). Use --force to delete anyway.",
    );
    strictEqual(result.hasUncommittedChanges, true);
    strictEqual(result.changedFiles, 2);
  });

  it("should delete dirty garden with --force", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git status --porcelain") {
          return Promise.resolve({
            stdout: " M file1.ts\n?? file2.ts\n",
            stderr: "",
          }); // Dirty status with 2 files
        }
        if (cmd.includes("git worktree remove")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git branch -D")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await deleteGarden("dirty-garden", { force: true });

    strictEqual(result.success, true);
    strictEqual(
      result.message,
      "Warning: Garden 'dirty-garden' had uncommitted changes (2 files)\nDeleted garden 'dirty-garden' and its branch 'phantom/gardens/dirty-garden'",
    );
    strictEqual(result.hasUncommittedChanges, true);
    strictEqual(result.changedFiles, 2);
  });

  it("should handle worktree remove failure and try force removal", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git status --porcelain") {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git worktree remove") && !cmd.includes("--force")) {
          return Promise.reject(new Error("Worktree remove failed"));
        }
        if (cmd.includes("git worktree remove --force")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git branch -D")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await deleteGarden("stubborn-garden");

    strictEqual(result.success, true);
    strictEqual(
      result.message,
      "Deleted garden 'stubborn-garden' and its branch 'phantom/gardens/stubborn-garden'",
    );
  });

  it("should handle case where branch doesn't exist", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git status --porcelain") {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git worktree remove")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git branch -D")) {
          return Promise.reject(new Error("Branch not found"));
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await deleteGarden("branch-missing-garden");

    strictEqual(result.success, true);
    strictEqual(
      result.message,
      "Deleted garden 'branch-missing-garden' and its branch 'phantom/gardens/branch-missing-garden'",
    );
  });

  it("should return error when force worktree removal also fails", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git status --porcelain") {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git worktree remove")) {
          return Promise.reject(new Error("Worktree removal failed"));
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await deleteGarden("impossible-garden");

    strictEqual(result.success, false);
    strictEqual(
      result.message,
      "Error: Failed to remove worktree for garden 'impossible-garden'",
    );
  });

  it("should handle git status errors gracefully", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock git commands
    execMock.mock.mockImplementation(
      (cmd: string, options?: { cwd?: string }) => {
        if (cmd === "git rev-parse --show-toplevel") {
          return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
        }
        if (cmd === "git status --porcelain") {
          return Promise.reject(new Error("Git status failed"));
        }
        if (cmd.includes("git worktree remove")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        if (cmd.includes("git branch -D")) {
          return Promise.resolve({ stdout: "", stderr: "" });
        }
        return Promise.resolve({ stdout: "", stderr: "" });
      },
    );

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await deleteGarden("status-error-garden");

    strictEqual(result.success, true);
    strictEqual(
      result.message,
      "Deleted garden 'status-error-garden' and its branch 'phantom/gardens/status-error-garden'",
    );
    strictEqual(result.hasUncommittedChanges, false);
  });
});
