import { execSync } from 'node:child_process';

export interface GitExecutor {
  execSync: typeof execSync;
}

export function getGitRoot(executor: GitExecutor = { execSync }): string {
  return executor.execSync('git rev-parse --show-toplevel', { encoding: 'utf8' }).trim();
}