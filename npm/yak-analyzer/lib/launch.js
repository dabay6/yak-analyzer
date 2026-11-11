import { dirname, join } from 'node:path';

/** The targets a platform package exists for, as Node names the operating system and the processor. */
export const TARGETS = ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64'];

/** The exit code when the launcher itself cannot run the binary. */
const FAILED = 1;

/**
 * The targets as a person reads them: the Linux ones are for glibc.
 *
 * @returns {string} The five targets, in one phrase.
 */
const targetsInWords = () => {
  return TARGETS.map((target) => {
    return target.startsWith('linux') ? `${target} (glibc)` : target;
  }).join(', ');
};

/**
 * The C library of a Linux machine, read from Node's own report without starting a process. Node's report names
 * the glibc version it runs on, and names none on a machine with another C library.
 *
 * @returns {'glibc' | 'musl' | undefined} The library; undefined off Linux.
 */
export function linuxLibc() {
  if (process.platform !== 'linux') {
    return undefined;
  }

  const excluded = process.report.excludeNetwork;

  // The report's list of network interfaces is slow to build and not needed.
  process.report.excludeNetwork = true;

  const { header } = process.report.getReport();

  process.report.excludeNetwork = excluded;

  return header.glibcVersionRuntime === undefined ? 'musl' : 'glibc';
}

/**
 * Finds the platform package's binary and runs it (spec PKG-5, PKG-6). The launcher answers no command itself: it
 * hands every argument on, and writes a line of its own only when it cannot run the binary.
 *
 * @param {object} options - The machine, and how to reach it.
 * @param {string} options.arch - The processor, as Node names it.
 * @param {readonly string[]} options.args - The arguments to hand on.
 * @param {string | undefined} options.libc - The C library on Linux.
 * @param {string} options.platform - The operating system, as Node names it.
 * @param {(id: string) => string} options.resolve - Resolves a module id to its file, and throws when it cannot.
 * @param {(file: string, args: readonly string[]) => { error?: Error, signal?: string | null, status?: number | null }} options.run -
 *   Runs the binary and waits for it.
 * @param {(line: string) => void} options.write - Writes a line to standard error.
 * @returns {number} The exit code: the binary's own, or 1 when it did not run to an end of its own.
 */
export function launch({ arch, args, libc, platform, resolve, run, write }) {
  const target = `${platform}-${arch}`;
  const otherLibc = platform === 'linux' && libc !== 'glibc';

  if (!TARGETS.includes(target) || otherLibc) {
    const machine = otherLibc ? `${target} (${libc ?? 'an unknown C library'})` : target;

    write(`yak-analyzer: there is no package for ${machine}; packages exist for ${targetsInWords()}\n`);

    return FAILED;
  }

  const name = `@dabay6/yak-analyzer-${target}`;
  let manifest;

  try {
    manifest = resolve(`${name}/package.json`);
  } catch {
    write(`yak-analyzer: the package ${name} is not installed; install again without leaving out optional dependencies\n`);

    return FAILED;
  }

  const file = join(dirname(manifest), platform === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer');
  const ended = run(file, args);

  if (ended.error !== undefined) {
    write(`yak-analyzer: cannot start ${file}: ${ended.error.message}\n`);

    return FAILED;
  }

  if (typeof ended.signal === 'string') {
    write(`yak-analyzer: ${file} was ended by ${ended.signal}\n`);

    return FAILED;
  }

  return ended.status ?? FAILED;
}
