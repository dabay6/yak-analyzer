import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { MAIN_DIR, manifest, ROOT } from './lib/run.mjs';

/** The README's text. */
const readme = readFileSync(join(ROOT, 'README.md'), 'utf8').replaceAll('\r\n', '\n');

test('the README says how to install and run the released binary (spec DOC-1)', () => {
  assert.ok(readme.includes('\n## Install\n'));
  assert.ok(readme.includes(`npm install ${manifest(MAIN_DIR).name}\n`));
  assert.ok(readme.includes('npx yak-analyzer --version\n'));
});

test('its status says the analyzer reports the five dead-code kinds, new or inherited (dead-code spec DOC-1)', () => {
  assert.ok(readme.includes('`yak-analyzer audit` reports the five dead-code kinds'));
  assert.ok(readme.includes('marks each finding new or unchanged'));
  assert.ok(readme.includes('`yak-analyzer graph` prints what it read'));
  assert.ok(!readme.includes('no findings yet'));
});

test('it names the oldest git the analyzer reads a project with, and a base in a partial clone (spec DEP-2, dead-code COMMIT-2)', () => {
  assert.ok(readme.includes('`git` 2.15 or later on the `PATH`'));
  assert.ok(readme.includes('a partial clone needs git 2.44 or later'));
});

test('it says what to remove on a machine that maps the scope to another registry (spec NAME-2a)', () => {
  assert.ok(readme.includes('npm config delete @dabay6:registry --location=user\n'));
});
