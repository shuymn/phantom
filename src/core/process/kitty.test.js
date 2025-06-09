import { strictEqual } from "node:assert";
import { describe, it } from "node:test";
import { isInsideKitty } from "./kitty.ts";

describe("kitty", () => {
  describe("isInsideKitty", () => {
    it("should return true when TERM is xterm-kitty", async () => {
      const originalTerm = process.env.TERM;
      const originalKittyWindowId = process.env.KITTY_WINDOW_ID;
      process.env.TERM = "xterm-kitty";
      // biome-ignore lint/performance/noDelete: Need to actually remove env var for test
      delete process.env.KITTY_WINDOW_ID;

      const result = await isInsideKitty();
      strictEqual(result, true);

      if (originalTerm === undefined) {
        // biome-ignore lint/performance/noDelete: Need to actually remove env var for test
        delete process.env.TERM;
      } else {
        process.env.TERM = originalTerm;
      }
      if (originalKittyWindowId !== undefined) {
        process.env.KITTY_WINDOW_ID = originalKittyWindowId;
      }
    });

    it("should return true when KITTY_WINDOW_ID is set", async () => {
      const originalTerm = process.env.TERM;
      const originalKittyWindowId = process.env.KITTY_WINDOW_ID;
      process.env.TERM = "xterm-256color";
      process.env.KITTY_WINDOW_ID = "1";

      const result = await isInsideKitty();
      strictEqual(result, true);

      if (originalTerm === undefined) {
        // biome-ignore lint/performance/noDelete: Need to actually remove env var for test
        delete process.env.TERM;
      } else {
        process.env.TERM = originalTerm;
      }
      if (originalKittyWindowId === undefined) {
        // biome-ignore lint/performance/noDelete: Need to actually remove env var for test
        delete process.env.KITTY_WINDOW_ID;
      } else {
        process.env.KITTY_WINDOW_ID = originalKittyWindowId;
      }
    });

    it("should return false when neither TERM=xterm-kitty nor KITTY_WINDOW_ID is set", async () => {
      const originalTerm = process.env.TERM;
      const originalKittyWindowId = process.env.KITTY_WINDOW_ID;
      process.env.TERM = "xterm-256color";
      // biome-ignore lint/performance/noDelete: Need to actually remove env var for test
      delete process.env.KITTY_WINDOW_ID;

      const result = await isInsideKitty();
      strictEqual(result, false);

      if (originalTerm === undefined) {
        // biome-ignore lint/performance/noDelete: Need to actually remove env var for test
        delete process.env.TERM;
      } else {
        process.env.TERM = originalTerm;
      }
      if (originalKittyWindowId !== undefined) {
        process.env.KITTY_WINDOW_ID = originalKittyWindowId;
      }
    });
  });
});
