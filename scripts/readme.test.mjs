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

test('its status says what the analyzer reads, and that it reports no findings yet (spec DOC-1)', () => {
  assert.ok(readme.includes('reports\nno findings yet'));
  assert.ok(readme.includes('`yak-analyzer graph` prints what it read'));
  assert.ok(!readme.includes('analyzes nothing yet'));
});

test('it names the oldest git the analyzer reads a project with (spec DEP-2)', () => {
  assert.ok(readme.includes('`git` 2.15 or later on the `PATH`'));
});

test('it says what to remove on a machine that maps the scope to another registry (spec NAME-2a)', () => {
  assert.ok(readme.includes('npm config delete @dabay6:registry --location=user\n'));
});
