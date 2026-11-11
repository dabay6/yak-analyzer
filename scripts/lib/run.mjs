import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/** The repository's root. */
export const ROOT = join(import.meta.dirname, '..', '..');

/** Where the packed files go. */
export const DIST = join(ROOT, 'dist');

/** This machine's target, as Node names the operating system and the processor. */
export const TARGET = `${process.platform}-${process.arch}`;

/** The main package's folder. */
export const MAIN_DIR = join(ROOT, 'npm', 'yak-analyzer');

/** The folder of this machine's platform package. */
export const PLATFORM_DIR = join(ROOT, 'npm', 'platforms', TARGET);

/** The binary's file name on this machine. */
export const BINARY = process.platform === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer';

/** The release build of the binary. */
export const BUILT = join(ROOT, 'target', 'release', BINARY);

/**
 * Reads a package's manifest.
 *
 * @param {string} dir - The package's folder.
 * @returns {{ bin?: Record<string, string>, name: string, version: string }} Its `package.json`.
 */
export function manifest(dir) {
  return JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
}

/**
 * The file `npm pack` writes for a package: its name without the `@`, with `/` as `-`, then its version.
 *
 * @param {string} dir - The package's folder.
 * @returns {string} The packed file, in `dist/`.
 */
export function packedFile(dir) {
  const { name, version } = manifest(dir);

  return join(DIST, `${name.replace('@', '').replace('/', '-')}-${version}.tgz`);
}

/**
 * Runs npm or npx and waits for it. Both are batch files on Windows, which Node starts only through a shell, and
 * the shell splits an argument at its spaces unless it is quoted.
 *
 * @param {'npm' | 'npx'} program - The program.
 * @param {readonly string[]} args - Its arguments.
 * @param {import('node:child_process').SpawnSyncOptions} [options] - Where to run it, and where its output goes.
 * @returns {import('node:child_process').SpawnSyncReturns<string>} How it ended.
 */
export function npm(program, args, options = {}) {
  const shell = process.platform === 'win32';
  const quoted = args.map((arg) => {
    return shell && arg.includes(' ') ? `"${arg}"` : arg;
  });

  return spawnSync(program, quoted, { encoding: 'utf8', shell, ...options });
}

/**
 * Stops the script with one line.
 *
 * @param {string} message - What went wrong, and what to do.
 * @returns {never}
 */
export function fail(message) {
  console.error(message);
  process.exit(1);
}
