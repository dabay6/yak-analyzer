import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The folder that holds the schema's copy and the note on where it came from. */
const FOLDER = join(ROOT, 'tests', 'hand-off');

/** A text file's content with line feeds, whatever the checkout wrote. */
const text = (name) => readFileSync(join(FOLDER, name), 'utf8').replaceAll('\r\n', '\n');

test('the copy of the hand-off schema is the file the note beside it describes (spec HO-1)', () => {
  const note = text('README.md');
  const stated = /SHA-256 of the file, with line feeds:\*\* `(?<digest>[0-9a-f]{64})`/u.exec(note)?.groups?.digest;
  const digest = createHash('sha256').update(text('hand-off-1.schema.json')).digest('hex');

  assert.equal(stated, digest);
});

test('the note says where the copy came from: the path, the pull request and the commit', () => {
  const note = text('README.md');

  assert.ok(note.includes('`packages/core/src/stacks/hand-off/hand-off-1.schema.json`'));
  assert.ok(note.includes('dabay6/yakshaver#213'));
  assert.match(note, /\*\*Commit:\*\* `[0-9a-f]{40}`/u);
});

test('the copy is hand-off format 1, as a JSON Schema', () => {
  const schema = JSON.parse(text('hand-off-1.schema.json'));

  assert.equal(schema.properties.formatVersion.const, 1);
  assert.deepEqual(schema.required, ['detection', 'formatVersion', 'root', 'siblings', 'stacks']);
});
