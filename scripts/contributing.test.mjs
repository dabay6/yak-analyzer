import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The contributing guide's text. */
const guide = readFileSync(join(ROOT, 'CONTRIBUTING.md'), 'utf8');

test('the guide says how a release is made, and what to do when one fails (spec PUB-12, PUB-14)', () => {
  for (const said of [
    '## Releasing',
    '### When a release fails',
    'docs.npmjs.com/trusted-publishers',
    'the environment `npm`',
    'the workflow file\n   `ci.yml`',
    '`NPM_TOKEN`',
    '"Bypass 2FA"',
    'refuse tokens',
  ]) {
    assert.ok(guide.replaceAll('\r\n', '\n').includes(said), said);
  }
});

test('the guide names the commands, and says how a fixture is written and run (spec DOC-1, OUT-2)', () => {
  for (const said of [
    '## The commands',
    '`yak-analyzer interface`',
    '`yak-analyzer graph --hand-off <file>`',
    'not part of the interface\nversion',
    '## Fixtures',
    '`tests/support/project.rs`',
    'cargo test --release --locked --test resolve',
  ]) {
    assert.ok(guide.replaceAll('\r\n', '\n').includes(said), said);
  }
});

test("the gates it names run the scripts' tests as well as the launcher's (spec GATE-2)", () => {
  assert.ok(guide.includes('node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"'));
});
