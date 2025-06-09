import { type Result, err, ok } from "../types/result.ts";
import { isObject } from "../utils/type-guards.ts";
import type { PhantomConfig } from "./loader.ts";

export class ConfigValidationError extends Error {
  constructor(message: string) {
    super(`Invalid phantom.config.json: ${message}`);
    this.name = "ConfigValidationError";
  }
}

export function validateConfig(
  config: unknown,
): Result<PhantomConfig, ConfigValidationError> {
  if (!isObject(config)) {
    return err(new ConfigValidationError("Configuration must be an object"));
  }

  const cfg = config;

  if (cfg.postCreate !== undefined) {
    if (!isObject(cfg.postCreate)) {
      return err(new ConfigValidationError("postCreate must be an object"));
    }

    const postCreate = cfg.postCreate;
    if (postCreate.copyFiles !== undefined) {
      if (!Array.isArray(postCreate.copyFiles)) {
        return err(
          new ConfigValidationError("postCreate.copyFiles must be an array"),
        );
      }

      if (!postCreate.copyFiles.every((f: unknown) => typeof f === "string")) {
        return err(
          new ConfigValidationError(
            "postCreate.copyFiles must contain only strings",
          ),
        );
      }
    }

    if (postCreate.commands !== undefined) {
      if (!Array.isArray(postCreate.commands)) {
        return err(
          new ConfigValidationError("postCreate.commands must be an array"),
        );
      }

      if (!postCreate.commands.every((c: unknown) => typeof c === "string")) {
        return err(
          new ConfigValidationError(
            "postCreate.commands must contain only strings",
          ),
        );
      }
    }
  }

  if (cfg.defaultMultiplexer !== undefined) {
    if (typeof cfg.defaultMultiplexer !== "string") {
      return err(
        new ConfigValidationError("defaultMultiplexer must be a string"),
      );
    }

    const validMultiplexers = ["tmux", "kitty", "none"];
    if (!validMultiplexers.includes(cfg.defaultMultiplexer)) {
      return err(
        new ConfigValidationError(
          `defaultMultiplexer must be one of: ${validMultiplexers.join(", ")}`,
        ),
      );
    }
  }

  return ok(config as PhantomConfig);
}
