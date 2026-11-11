/**
 * Installs the packed main package and this machine's packed platform package into a new folder, the way npm and
 * yak will, and runs the launcher's version answer (spec PKG-9). The install fetches nothing and runs no install
 * script: the four other platform packages are simply absent, as they are on a user's machine.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { fail, MAIN_DIR, manifest, npm, packedFile, PLATFORM_DIR } from './lib/run.mjs';

/** The two packed files. */
const files = [packedFile(MAIN_DIR), packedFile(PLATFORM_DIR)];

for (const file of files) {
  if (!existsSync(file)) {
    fail(`${file} is not packed; run scripts/pack.mjs first`);
  }
}

/** The main package's name, version and `bin` map. */
const { bin, name, version } = manifest(MAIN_DIR);

/** The line the binary prints for its version (spec BIN-1). */
const expected = `yak-analyzer ${version}\n`;

/** A new folder to install into. */
const folder = mkdtempSync(join(tmpdir(), 'yak-analyzer-install-'));

/** What went wrong, if anything did. */
let problem;

try {
  // The folder needs a manifest of its own, or npm installs into the nearest folder above that has one.
  writeFileSync(join(folder, 'package.json'), '{"private":true}\n');

  const installed = npm(
    'npm',
    ['install', '--offline', '--ignore-scripts', '--no-audit', '--no-fund', '--no-update-notifier', ...files],
    { cwd: folder, stdio: 'inherit' }
  );

  if (installed.status === 0) {
    // yak starts a tool by running its `bin` file with Node, through no wrapper script.
    const answered = spawnSync(process.execPath, [join(folder, 'node_modules', name, bin['yak-analyzer']), '--version'], {
      encoding: 'utf8',
    });

    if (answered.status !== 0 || answered.stdout !== expected) {
      problem = `the installed launcher answered ${JSON.stringify(answered.stdout)} and exited ${answered.status} (${answered.stderr.trim()}); expected ${JSON.stringify(expected)}`;
    }
  } else {
    problem = 'npm could not install the packed files';
  }
} finally {
  rmSync(folder, { force: true, recursive: true });
}

if (problem !== undefined) {
  fail(problem);
}

console.log(`the packed install answers ${expected.trim()}`);
