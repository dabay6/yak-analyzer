import assert from 'node:assert/strict';
import { test } from 'node:test';

import { coldKind, columns, combine, median, STAGES, table, timingLine } from './lib/measure.mjs';

/**
 * A run, as the script records one.
 *
 * @param {string} kind - Its kind.
 * @param {number} wall - Its wall-clock time, in microseconds.
 * @param {number} each - The time of every stage, in microseconds.
 * @returns {import('./lib/measure.mjs').Run} The run.
 */
const run = (kind, wall, each) => {
  const reading = Object.fromEntries(
    STAGES.map((stage) => {
      return [stage, each];
    })
  );

  return { kind, timings: { print: 100, reading, total: each * STAGES.length + 300 }, wall };
};

/**
 * A measured document, as `--json` writes one.
 *
 * @param {import('./lib/measure.mjs').Run[]} runs - Its runs.
 * @param {Partial<import('./lib/measure.mjs').Measured>} [changed] - The fields that differ from the default.
 * @returns {import('./lib/measure.mjs').Measured} The document.
 */
const measured = (runs, changed = {}) => {
  return {
    commit: 'abc',
    dirty: false,
    exeHash: 'hash',
    folder: '/stand-in',
    label: null,
    platform: 'linux-x64',
    runs,
    version: 'yak-analyzer 0.3.0',
    ...changed,
  };
};

test('finds the timing line among the lines on standard error (spec TIME-5)', () => {
  const line = JSON.stringify({ timings: { print: 2, reading: { files: 1 }, total: 9 } });

  assert.deepEqual(timingLine(`something else\r\n${line}\n`), { print: 2, reading: { files: 1 }, total: 9 });
});

test('gives nothing when no timing line came, so the run is not averaged in (spec MEAS-5)', () => {
  assert.equal(timingLine(''), undefined);
  assert.equal(timingLine('yak-analyzer: the file a.ts does not parse\n'), undefined);
});

test('takes the middle value, or the mean of the two middle values', () => {
  assert.equal(median([5, 1, 3]), 3);
  assert.equal(median([4, 1, 3, 2]), 2.5);
});

test("shows the time no stage covers as other, and process start as wall minus total (spec TIME-2, MEAS-3)", () => {
  const shown = columns(run('warm', 5000, 10));

  assert.equal(shown.files, 10);
  assert.equal(shown.print, 100);
  assert.equal(shown.total, 10 * STAGES.length + 300);
  assert.equal(shown.other, 200);
  assert.equal(shown.start, 5000 - (10 * STAGES.length + 300));
  assert.equal(shown.wall, 5000);
});

test('labels the first run cold only on the caller\'s word or with the cache emptied (spec MEAS-4)', () => {
  assert.equal(coldKind({ cold: true, dropCaches: false }), 'cold');
  assert.equal(coldKind({ cold: false, dropCaches: true }), 'cold');
  assert.equal(coldKind({ cold: false, dropCaches: false }), 'unknown');
});

test('pools the runs of documents that agree (spec MEAS-3a)', () => {
  const combined = combine([measured([run('cold', 900, 50)]), measured([run('cold', 800, 40), run('warm', 100, 5)])]);

  assert.equal(combined.runs.length, 3);
  assert.equal(combined.platform, 'linux-x64');
});

test('refuses documents that differ, naming the difference (spec MEAS-3a)', () => {
  for (const field of ['platform', 'exeHash', 'commit', 'dirty', 'label']) {
    const changed = { [field]: field === 'dirty' ? true : 'other' };

    assert.throws(
      () => {
        return combine([measured([]), measured([], changed)]);
      },
      { message: new RegExp(`differ in ${field}`, 'u') },
      field
    );
  }

  assert.throws(
    () => {
      return combine([]);
    },
    { message: /nothing to combine/u }
  );
});

test('fills the cold minus warm column from cold runs only (spec MEAS-3a)', () => {
  const shown = table(measured([run('cold', 9000, 500), run('warm', 1000, 100), run('warm', 1200, 100)]));
  const files = shown.split('\n').find((line) => {
    return line.startsWith('| files |');
  });

  assert.ok(files?.endsWith('| 0.4 |'), files);
});

test('reads none in the cold columns when there are no cold runs, and never pools unknown runs with cold ones', () => {
  const shown = table(measured([run('unknown', 9000, 500), run('warm', 1000, 100)]));
  const files = shown.split('\n').find((line) => {
    return line.startsWith('| files |');
  });

  assert.ok(files?.startsWith('| files | none | 0.5 (0.5 to 0.5) | 0.1 (0.1 to 0.1) |'), files);
  assert.ok(files?.endsWith('| none |'), files);
});
