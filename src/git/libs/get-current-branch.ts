import { execSync } from 'node:child_process';

export interface GitExecutor {
  execSync: typeof execSync;
}

export function getCurrentBranch(executor: GitExecutor = { execSync }): string {
  return executor.execSync('git branch --show-current', { encoding: 'utf8' }).trim();
}