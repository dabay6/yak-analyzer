import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readdirSync } from 'node:fs';
import { basename, join } from 'node:path';
import { test } from 'node:test';

import { MAIN_DIR, PACKAGE_DIRS, PUBLISHER, REGISTRY, ROOT, TARGETS } from './lib/run.mjs';

/** The script that publishes, and tries the publishing. */
const SCRIPT = join(ROOT, 'scripts', 'dry-run-publish.mjs');

/**
 * Runs the script with the environment given, and no marker of the publishing job unless it is given one.
 *
 * @param {readonly string[]} args - The script's arguments.
 * @param {Record<string, string>} [env] - What to add to the environment.
 * @returns {import('node:child_process').SpawnSyncReturns<string>} How it ended.
 */
const script = (args, env = {}) => {
  const { YAK_ANALYZER_PUBLISHING_JOB: _marker, ...rest } = process.env;

  return spawnSync(process.execPath, [SCRIPT, ...args], { encoding: 'utf8', env: { ...rest, ...env } });
};

test('the publishing way is refused outside the publishing job, before anything is read or sent (spec CI-5)', () => {
  const refused = script(['--all', '--publish']);

  assert.equal(refused.status, 1);
  assert.equal(refused.stdout, '');
  assert.equal(refused.stderr, 'only the publishing job of the workflow publishes; run this script without --publish for a dry run\n');
});

test('a marker that is not exactly the publishing job\'s is refused too', () => {
  assert.equal(script(['--publish'], { YAK_ANALYZER_PUBLISHING_JOB: '1' }).status, 1);
});

test('an argument the script does not know stops it', () => {
  const stopped = script(['--force']);

  assert.equal(stopped.status, 1);
  assert.match(stopped.stderr, /^unknown argument --force; /u);
});

test('the packages are published platform packages first and the main package last (spec PUB-4)', () => {
  assert.equal(PACKAGE_DIRS.length, 6);
  assert.equal(PACKAGE_DIRS.at(-1), MAIN_DIR);
  assert.deepEqual(PACKAGE_DIRS.slice(0, 5).map((dir) => basename(dir)), TARGETS);
});

test('the targets are the folders under npm/platforms', () => {
  assert.deepEqual([...TARGETS].sort(), readdirSync(join(ROOT, 'npm', 'platforms')).sort());
});

test('the npm that publishes is one exact version of 11.5.1 or later that runs on Node 22.18 (spec CI-5)', () => {
  const [, major, minor, patch] = /^npm@(\d+)\.(\d+)\.(\d+)$/u.exec(PUBLISHER).map(Number);

  assert.equal(major, 11);
  assert.ok(minor > 5 || (minor === 5 && patch >= 1));
});

test('every command names npmjs.com as the registry of the scope', () => {
  assert.equal(REGISTRY, '--@dabay6:registry=https://registry.npmjs.org/');
});
