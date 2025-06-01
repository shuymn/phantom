import { exit } from 'node:process';
import { execSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

export function ruinsCreateHandler(args: string[]): void {
  const name = args[0];
  if (!name) {
    console.error('Error: ruin name required');
    exit(1);
  }
  
  try {
    const gitRoot = execSync('git rev-parse --show-toplevel', { encoding: 'utf8' }).trim();
    const ruinsPath = join(gitRoot, '.git', 'phantom', 'ruins');
    const worktreePath = join(ruinsPath, name);
    
    if (!existsSync(ruinsPath)) {
      mkdirSync(ruinsPath, { recursive: true });
    }
    
    if (existsSync(worktreePath)) {
      console.error(`Error: ruin '${name}' already exists`);
      exit(1);
    }
    
    const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
    
    execSync(`git worktree add "${worktreePath}" -b "phantom/ruins/${name}" HEAD`, { stdio: 'inherit' });
    
    console.log(`Created ruin '${name}' at ${worktreePath}`);
  } catch (error) {
    console.error('Error creating ruin:', error.message);
    exit(1);
  }
}