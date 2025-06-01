import { execSync, type ExecSyncOptions } from 'node:child_process';

export interface WorktreeExecutor {
  execSync: typeof execSync;
}

export interface AddWorktreeOptions {
  path: string;
  branch: string;
  commitish?: string;
}

export function addWorktree(
  options: AddWorktreeOptions,
  executor: WorktreeExecutor = { execSync }
): void {
  const { path, branch, commitish = 'HEAD' } = options;
  const execOptions: ExecSyncOptions = { stdio: 'inherit' };
  
  executor.execSync(
    `git worktree add "${path}" -b "${branch}" ${commitish}`,
    execOptions
  );
}