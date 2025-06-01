import { strictEqual } from "node:assert";
import { before, describe, it, mock } from "node:test";

describe("whereGarden", () => {
  let accessMock: ReturnType<typeof mock.fn>;
  let execMock: ReturnType<typeof mock.fn>;
  let whereGarden: typeof import("./where.ts").whereGarden;

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

    ({ whereGarden } = await import("./where.ts"));
  });

  it("should return error when name is not provided", async () => {
    const result = await whereGarden("");
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

    const result = await whereGarden("nonexistent-garden");

    strictEqual(result.success, false);
    strictEqual(
      result.message,
      "Error: Garden 'nonexistent-garden' does not exist",
    );
  });

  it("should return path when garden exists", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await whereGarden("existing-garden");

    strictEqual(result.success, true);
    strictEqual(result.path, "/test/repo/.git/phantom/gardens/existing-garden");
  });

  it("should handle git root detection failures", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot failure
    execMock.mock.mockImplementation(() => {
      return Promise.reject(new Error("Not a git repository"));
    });

    const result = await whereGarden("some-garden");

    strictEqual(result.success, false);
    strictEqual(result.message, "Error locating garden: Not a git repository");
  });

  it("should handle different garden names correctly", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/different/repo\n", stderr: "" });
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await whereGarden("feature-branch-123");

    strictEqual(result.success, true);
    strictEqual(
      result.path,
      "/different/repo/.git/phantom/gardens/feature-branch-123",
    );
  });

  it("should handle special characters in garden names", async () => {
    accessMock.mock.resetCalls();
    execMock.mock.resetCalls();

    // Mock getGitRoot
    execMock.mock.mockImplementation((cmd: string) => {
      if (cmd === "git rev-parse --show-toplevel") {
        return Promise.resolve({ stdout: "/test/repo\n", stderr: "" });
      }
      return Promise.resolve({ stdout: "", stderr: "" });
    });

    // Mock garden exists
    accessMock.mock.mockImplementation(() => Promise.resolve());

    const result = await whereGarden("feature-with-dashes_and_underscores");

    strictEqual(result.success, true);
    strictEqual(
      result.path,
      "/test/repo/.git/phantom/gardens/feature-with-dashes_and_underscores",
    );
  });
});
