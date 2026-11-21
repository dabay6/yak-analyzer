import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The folder that holds the schema's copy and the note on where it came from. */
const FOLDER = join(ROOT, 'tests', 'sarif');

/** A text file's content with line feeds, whatever the checkout wrote. */
const text = (name) => readFileSync(join(FOLDER, name), 'utf8').replaceAll('\r\n', '\n');

test('the copy of the SARIF schema is the file the note beside it describes (spec FIX-3)', () => {
  const note = text('README.md');
  const stated = /SHA-256 of the file, with line feeds:\*\* `(?<digest>[0-9a-f]{64})`/u.exec(note)?.groups?.digest;
  const digest = createHash('sha256').update(text('sarif-schema-2.1.0.json')).digest('hex');

  assert.equal(stated, digest);
});

test('the note says where the copy came from: the address and the edition', () => {
  const note = text('README.md');
  const schema = JSON.parse(text('sarif-schema-2.1.0.json'));

  assert.ok(note.includes(`\`${schema.id}\``));
  assert.ok(note.includes('OASIS Standard, Errata 01'));
});

test('the copy is the schema of SARIF 2.1.0', () => {
  const schema = JSON.parse(text('sarif-schema-2.1.0.json'));

  assert.deepEqual(schema.properties.version.enum, ['2.1.0']);
  assert.deepEqual(schema.required, ['version', 'runs']);
});
