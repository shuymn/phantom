import { exit } from 'node:process';
import { existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { getGitRoot, type GitExecutor } from '../../git/libs/get-git-root.ts';
import { addWorktree, type WorktreeExecutor } from '../../git/libs/add-worktree.ts';

export interface RuinsCreateOptions {
  gitExecutor?: GitExecutor;
  worktreeExecutor?: WorktreeExecutor;
  existsSync?: typeof existsSync;
  mkdirSync?: typeof mkdirSync;
  exit?: typeof exit;
  console?: Pick<Console, 'log' | 'error'>;
}

export function createRuin(
  name: string, 
  options: RuinsCreateOptions = {}
): { success: boolean; message: string; path?: string } {
  const {
    gitExecutor,
    worktreeExecutor,
    existsSync: exists = existsSync,
    mkdirSync: mkdir = mkdirSync,
  } = options;

  if (!name) {
    return { success: false, message: 'Error: ruin name required' };
  }
  
  try {
    const gitRoot = getGitRoot(gitExecutor);
    const ruinsPath = join(gitRoot, '.git', 'phantom', 'ruins');
    const worktreePath = join(ruinsPath, name);
    
    if (!exists(ruinsPath)) {
      mkdir(ruinsPath, { recursive: true });
    }
    
    if (exists(worktreePath)) {
      return { success: false, message: `Error: ruin '${name}' already exists` };
    }
    
    addWorktree({
      path: worktreePath,
      branch: `phantom/ruins/${name}`,
      commitish: 'HEAD'
    }, worktreeExecutor);
    
    return { 
      success: true, 
      message: `Created ruin '${name}' at ${worktreePath}`,
      path: worktreePath
    };
  } catch (error) {
    return { success: false, message: `Error creating ruin: ${error.message}` };
  }
}

export function ruinsCreateHandler(args: string[]): void {
  const name = args[0];
  const result = createRuin(name);
  
  if (!result.success) {
    console.error(result.message);
    exit(1);
  }
  
  console.log(result.message);
}