import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { test } from 'node:test';

import { launch, linuxLibc, TARGETS } from '#yak-analyzer/launch.js';

/** A folder that stands for an installed platform package. */
const INSTALLED = join('made-up', 'node_modules', '@dabay6', 'yak-analyzer-linux-x64');

/**
 * Runs the launcher on a made-up machine, keeping what it writes.
 *
 * @param {object} machine - What to change from a Linux x64 machine with glibc and the package installed.
 * @returns {{ code: number, lines: string[], ran: unknown[] }} The exit code, the lines written, and each run.
 */
const onMachine = (machine = {}) => {
  const lines = [];
  const ran = [];
  const code = launch({
    arch: 'x64',
    args: ['--version'],
    libc: 'glibc',
    platform: 'linux',
    resolve: () => join(INSTALLED, 'package.json'),
    run: (file, args) => {
      ran.push({ args, file });

      return { signal: null, status: 0 };
    },
    write: (line) => lines.push(line),
    ...machine,
  });

  return { code, lines, ran };
};

test('the targets are the five the packages exist for', () => {
  assert.deepEqual(TARGETS, ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64']);
});

test('runs the platform package binary with the arguments it was given, and says nothing itself', () => {
  const { code, lines, ran } = onMachine();

  assert.equal(code, 0);
  assert.deepEqual(lines, []);
  assert.deepEqual(ran, [{ args: ['--version'], file: join(INSTALLED, 'yak-analyzer') }]);
});

test('the binary has the Windows name on Windows', () => {
  const { ran } = onMachine({ platform: 'win32' });

  assert.equal(ran[0].file, join(INSTALLED, 'yak-analyzer.exe'));
});

test('exits with the exit code of a real program that stands in for the binary', () => {
  const script = 'process.stdout.write(process.argv.slice(1).join(","));process.exit(7)';
  let output = '';
  const { code, lines } = onMachine({
    args: ['one', 'two'],
    run: (_file, args) => {
      const child = spawnSync(process.execPath, ['-e', script, '--', ...args], { encoding: 'utf8' });

      output = child.stdout;

      return child;
    },
  });

  assert.equal(code, 7);
  assert.equal(output, 'one,two');
  assert.deepEqual(lines, []);
});

test('names the targets that have a package where the machine has none', () => {
  const { code, lines, ran } = onMachine({ arch: 'x64', platform: 'freebsd' });

  assert.equal(code, 1);
  assert.deepEqual(ran, []);
  assert.equal(lines.length, 1);
  assert.match(lines[0], /^yak-analyzer: there is no package for freebsd-x64; /u);

  for (const target of TARGETS) {
    assert.ok(lines[0].includes(target), target);
  }
});

test('treats Linux with another C library as a machine with no package, and says which library', () => {
  const { code, lines, ran } = onMachine({ libc: 'musl' });

  assert.equal(code, 1);
  assert.deepEqual(ran, []);
  assert.match(lines[0], /^yak-analyzer: there is no package for linux-x64 \(musl\); /u);
  assert.ok(lines[0].includes('linux-x64 (glibc)'));
});

test('says the package is missing, and why it may be, where one exists and is not installed', () => {
  const { code, lines, ran } = onMachine({
    resolve: () => {
      throw new Error('Cannot find module');
    },
  });

  assert.equal(code, 1);
  assert.deepEqual(ran, []);
  assert.equal(lines.length, 1);
  assert.ok(lines[0].includes('@dabay6/yak-analyzer-linux-x64 is not installed'));
  assert.ok(lines[0].includes('optional dependencies'));
});

test('names the file and the reason when the binary is there and cannot be started', () => {
  const { code, lines } = onMachine({ run: (file, args) => spawnSync(file, args) });

  assert.equal(code, 1);
  assert.equal(lines.length, 1);
  assert.ok(lines[0].startsWith(`yak-analyzer: cannot start ${join(INSTALLED, 'yak-analyzer')}: `));
});

test('names the signal when the binary was ended by one', () => {
  const { code, lines } = onMachine({ run: () => ({ signal: 'SIGKILL', status: null }) });

  assert.equal(code, 1);
  assert.deepEqual(lines, [`yak-analyzer: ${join(INSTALLED, 'yak-analyzer')} was ended by SIGKILL\n`]);
});

test('every line the launcher writes is one line', () => {
  for (const machine of [{ platform: 'freebsd' }, { libc: 'musl' }, { run: () => ({ signal: 'SIGTERM' }) }]) {
    const { lines } = onMachine(machine);

    assert.equal(lines.join('').trimEnd().split('\n').length, 1);
  }
});

test('reads the C library only on Linux', () => {
  if (process.platform === 'linux') {
    assert.ok(['glibc', 'musl'].includes(linuxLibc()));
  } else {
    assert.equal(linuxLibc(), undefined);
  }
});
