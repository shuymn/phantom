import { describe, it, mock } from 'node:test';
import { strictEqual, deepStrictEqual } from 'node:assert';
import { createRuin } from './create.ts';

describe('createRuin', () => {
  it('should return error when name is not provided', () => {
    const result = createRuin('');
    strictEqual(result.success, false);
    strictEqual(result.message, 'Error: ruin name required');
  });

  it('should create ruin directory when it does not exist', () => {
    const mkdirSyncMock = mock.fn();
    const execSyncMock = mock.fn((cmd: string) => {
      if (cmd === 'git rev-parse --show-toplevel') {
        return '/test/repo\n';
      }
      return '';
    });
    const existsSyncMock = mock.fn(() => false);

    const result = createRuin('test-ruin', {
      execSync: execSyncMock as any,
      existsSync: existsSyncMock as any,
      mkdirSync: mkdirSyncMock as any,
    });

    strictEqual(result.success, true);
    strictEqual(result.message, 'Created ruin \'test-ruin\' at /test/repo/.git/phantom/ruins/test-ruin');
    strictEqual(result.path, '/test/repo/.git/phantom/ruins/test-ruin');
    
    strictEqual(mkdirSyncMock.mock.calls.length, 1);
    deepStrictEqual(mkdirSyncMock.mock.calls[0].arguments, [
      '/test/repo/.git/phantom/ruins',
      { recursive: true }
    ]);
    
    strictEqual(execSyncMock.mock.calls.length, 2);
    strictEqual(execSyncMock.mock.calls[0].arguments[0], 'git rev-parse --show-toplevel');
    strictEqual(execSyncMock.mock.calls[1].arguments[0], 'git worktree add "/test/repo/.git/phantom/ruins/test-ruin" -b "phantom/ruins/test-ruin" HEAD');
  });

  it('should return error when ruin already exists', () => {
    const execSyncMock = mock.fn((cmd: string) => {
      if (cmd === 'git rev-parse --show-toplevel') {
        return '/test/repo\n';
      }
      return '';
    });
    const existsSyncMock = mock.fn((path: string) => {
      if (path === '/test/repo/.git/phantom/ruins') return true;
      if (path === '/test/repo/.git/phantom/ruins/existing-ruin') return true;
      return false;
    });

    const result = createRuin('existing-ruin', {
      execSync: execSyncMock as any,
      existsSync: existsSyncMock as any,
    });

    strictEqual(result.success, false);
    strictEqual(result.message, 'Error: ruin \'existing-ruin\' already exists');
  });

  it('should handle git command errors', () => {
    const execSyncMock = mock.fn(() => {
      throw new Error('Not a git repository');
    });

    const result = createRuin('test-ruin', {
      execSync: execSyncMock as any,
    });

    strictEqual(result.success, false);
    strictEqual(result.message, 'Error creating ruin: Not a git repository');
  });

  it('should not create ruins directory if it already exists', () => {
    const mkdirSyncMock = mock.fn();
    const execSyncMock = mock.fn((cmd: string) => {
      if (cmd === 'git rev-parse --show-toplevel') {
        return '/test/repo\n';
      }
      return '';
    });
    const existsSyncMock = mock.fn((path: string) => {
      return path === '/test/repo/.git/phantom/ruins';
    });

    const result = createRuin('test-ruin', {
      execSync: execSyncMock as any,
      existsSync: existsSyncMock as any,
      mkdirSync: mkdirSyncMock as any,
    });

    strictEqual(result.success, true);
    strictEqual(mkdirSyncMock.mock.calls.length, 0);
  });
});