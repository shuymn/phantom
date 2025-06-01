import { exit } from 'node:process';
import { execSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

export interface RuinsCreateOptions {
  execSync?: typeof execSync;
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
    execSync: exec = execSync,
    existsSync: exists = existsSync,
    mkdirSync: mkdir = mkdirSync,
  } = options;

  if (!name) {
    return { success: false, message: 'Error: ruin name required' };
  }
  
  try {
    const gitRoot = exec('git rev-parse --show-toplevel', { encoding: 'utf8' }).trim();
    const ruinsPath = join(gitRoot, '.git', 'phantom', 'ruins');
    const worktreePath = join(ruinsPath, name);
    
    if (!exists(ruinsPath)) {
      mkdir(ruinsPath, { recursive: true });
    }
    
    if (exists(worktreePath)) {
      return { success: false, message: `Error: ruin '${name}' already exists` };
    }
    
    exec(`git worktree add "${worktreePath}" -b "phantom/ruins/${name}" HEAD`, { stdio: 'inherit' });
    
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