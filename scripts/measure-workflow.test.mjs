import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The measuring workflow's text, read as text as `workflow.test.mjs` reads `ci.yml`. */
const workflow = readFileSync(join(ROOT, '.github', 'workflows', 'measure.yml'), 'utf8').replaceAll('\r\n', '\n');

test('it runs only when started by hand (spec JOB-1)', () => {
  assert.match(workflow, /\non:\n {2}workflow_dispatch:\n/u);
  assert.doesNotMatch(workflow, /pull_request|\n {2}push:/u);
});

test('it runs straight on the Linux runner ci.yml uses, with no container (spec JOB-1)', () => {
  assert.match(workflow, /\n {4}runs-on: ubuntu-24\.04\n/u);
  assert.doesNotMatch(workflow, /container:/u);
});

test('it measures the stand-in at one commit pinned by its full hash (spec GEN-1, GEN-2)', () => {
  assert.match(workflow, /STAND_IN: mui\/base-ui\n/u);
  assert.match(workflow, /STAND_IN_COMMIT: [0-9a-f]{40}\n/u);
  assert.match(workflow, /fetch --quiet --depth 1 "https:\/\/github\.com\/\$STAND_IN" "\$STAND_IN_COMMIT"/u);
});

test('it builds, measures three rounds with the cache emptied, and combines them (spec JOB-2)', () => {
  assert.ok(workflow.includes('cargo build --release --locked'));
  assert.ok(workflow.includes('for round in 1 2 3; do'));
  assert.ok(workflow.includes('--drop-caches --json > "measured/round-$round.json"'));
  assert.ok(
    workflow.includes(
      'node scripts/measure.mjs --combine measured/round-1.json measured/round-2.json measured/round-3.json >> "$GITHUB_STEP_SUMMARY"'
    )
  );
});

test('it keeps the rounds as an artifact, with the actions pinned as ci.yml pins them (spec JOB-3)', () => {
  const ci = readFileSync(join(ROOT, '.github', 'workflows', 'ci.yml'), 'utf8');

  for (const action of [/actions\/checkout@[0-9a-f]{40}/u, /actions\/setup-node@[0-9a-f]{40}/u, /actions\/upload-artifact@[0-9a-f]{40}/u]) {
    const pinned = workflow.match(action)?.[0];

    assert.ok(pinned !== undefined && ci.includes(pinned), String(action));
  }

  assert.match(workflow, /name: measured\n {10}path: measured\/\*\.json\n/u);
});
