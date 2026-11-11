import { spawnSync } from 'node:child_process';
import { appendFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

/** The repository's root. */
export const ROOT = join(import.meta.dirname, '..', '..');

/** Where the packed files go. */
export const DIST = join(ROOT, 'dist');

/** This machine's target, as Node names the operating system and the processor. */
export const TARGET = `${process.platform}-${process.arch}`;

/** The targets a platform package exists for. */
export const TARGETS = ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64'];

/** The main package's folder. */
export const MAIN_DIR = join(ROOT, 'npm', 'yak-analyzer');

/** The folder of this machine's platform package. */
export const PLATFORM_DIR = join(ROOT, 'npm', 'platforms', TARGET);

/**
 * Every package's folder in the order the packages are published: the platform packages first and the main
 * package last, so a main package never exists without its platform packages (spec PUB-4).
 */
export const PACKAGE_DIRS = [
  ...TARGETS.map((target) => {
    return join(ROOT, 'npm', 'platforms', target);
  }),
  MAIN_DIR,
];

/** The binary's file name on this machine. */
export const BINARY = process.platform === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer';

/** The release build of the binary. */
export const BUILT = join(ROOT, 'target', 'release', BINARY);

/**
 * The npm that publishes, fetched by `npx` at this exact version (spec CI-5). Trusted publishing needs 11.5.1 or
 * later, and npm 12 does not run on Node 22.18. It is moved by hand.
 */
export const PUBLISHER = 'npm@11.21.2';

/** Where the packages are published, said on each command so that a machine's own settings cannot change it. */
export const REGISTRY = '--@dabay6:registry=https://registry.npmjs.org/';

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

/**
 * Asks npmjs.com whether a package's version is there for anyone to install. npm answers a version that is not
 * there, and a package that is not there, with the code E404; anything else is a failure of the question itself.
 *
 * @param {string} name - The package.
 * @param {string} version - The version.
 * @returns {boolean} True when it is published.
 */
export function isPublished(name, version) {
  const asked = npm('npx', ['--yes', PUBLISHER, 'view', `${name}@${version}`, 'version', '--json', REGISTRY]);

  if (asked.status === 0) {
    return true;
  }

  let code;

  try {
    code = JSON.parse(asked.stdout).error?.code;
  } catch {
    code = undefined;
  }

  if (code !== 'E404') {
    fail(`npmjs.com could not be asked about ${name}@${version}: ${`${asked.stderr}${asked.stdout}`.trim()}`);
  }

  return false;
}

/**
 * Waits, without starting a process.
 *
 * @param {number} seconds - How long.
 */
export function sleep(seconds) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, seconds * 1000);
}

/**
 * Hands a value to the workflow's later jobs. It does nothing outside a workflow.
 *
 * @param {string} name - The output's name.
 * @param {string} value - Its value, on one line.
 */
export function output(name, value) {
  console.log(`${name}=${value}`);

  if (process.env.GITHUB_OUTPUT !== undefined) {
    appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
  }
}
