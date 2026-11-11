/**
 * Installs the published main package by its name and version from npmjs.com into a new folder, as a user's npm
 * will, and runs the launcher's version answer (spec PUB-9). It then reads from npmjs.com that the main package
 * and this machine's platform package carry provenance at that version (spec PUB-15). It needs no secret.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { fail, MAIN_DIR, manifest, npm, PLATFORM_DIR, PUBLISHER, REGISTRY, sleep } from './lib/run.mjs';

/** How many times, half a minute apart, the install is tried while npmjs.com catches up with a publish. */
const TRIES = 10;

/** The main package's name, version and `bin` map. */
const { bin, name, version } = manifest(MAIN_DIR);

/** This machine's platform package. */
const platform = manifest(PLATFORM_DIR).name;

/** The line the binary prints for its version (spec BIN-1). */
const expected = `yak-analyzer ${version}\n`;

/** A new folder to install into. */
const folder = mkdtempSync(join(tmpdir(), 'yak-analyzer-published-'));

/** What went wrong, if anything did. */
let problem;

try {
  // The folder needs a manifest of its own, or npm installs into the nearest folder above that has one.
  writeFileSync(join(folder, 'package.json'), '{"private":true}\n');

  let installed = false;

  for (let i = 0; i < TRIES && !installed; i += 1) {
    if (i > 0) {
      sleep(30);
    }

    installed =
      npm('npm', ['install', '--ignore-scripts', '--no-audit', '--no-fund', '--no-update-notifier', REGISTRY, `${name}@${version}`], {
        cwd: folder,
        stdio: 'inherit',
      }).status === 0;
  }

  if (!installed) {
    problem = `npm could not install ${name}@${version} from npmjs.com`;
  } else if (existsSync(join(folder, 'node_modules', platform))) {
    // yak starts a tool by running its `bin` file with Node, through no wrapper script.
    const answered = spawnSync(process.execPath, [join(folder, 'node_modules', name, bin['yak-analyzer']), '--version'], {
      encoding: 'utf8',
    });

    if (answered.status !== 0 || answered.stdout !== expected) {
      problem = `the installed launcher answered ${JSON.stringify(answered.stdout)} and exited ${answered.status} (${answered.stderr.trim()}); expected ${JSON.stringify(expected)}`;
    }
  } else {
    problem = `npm installed ${name}@${version} without ${platform}`;
  }
} finally {
  rmSync(folder, { force: true, recursive: true });
}

if (problem !== undefined) {
  fail(problem);
}

console.log(`${name}@${version}, installed from npmjs.com, answers ${expected.trim()}`);

for (const published of [name, platform]) {
  const asked = npm('npx', ['--yes', PUBLISHER, 'view', `${published}@${version}`, 'dist.attestations', '--json', REGISTRY]);
  let kind;

  try {
    kind = JSON.parse(asked.stdout).provenance?.predicateType;
  } catch {
    kind = undefined;
  }

  // npm prints nothing for a version that carries no provenance.
  if (asked.status !== 0 || typeof kind !== 'string' || !kind.startsWith('https://slsa.dev/provenance/')) {
    fail(`${published}@${version} carries no provenance on npmjs.com: ${`${asked.stdout}${asked.stderr}`.trim()}`);
  }

  console.log(`${published}@${version} carries provenance`);
}
