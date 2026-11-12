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

test('its status says what the release is: a binary that answers its version and analyzes nothing yet', () => {
  assert.ok(readme.includes('answers its version and analyzes nothing yet'));
  assert.ok(!readme.includes('there is no\nrelease'));
});

test('it says what to remove on a machine that maps the scope to another registry (spec NAME-2a)', () => {
  assert.ok(readme.includes('npm config delete @dabay6:registry --location=user\n'));
});
