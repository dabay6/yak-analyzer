/**
 * What the measuring script reads, works out and prints, kept apart from running anything so that it can be
 * tested on its own (spec MEAS-3, MEAS-3a, MEAS-5, MEAS-6).
 */

/** @typedef {{ print: number, reading: Record<string, number>, total: number }} Timings */

/** @typedef {{ kind: string, timings: Timings, wall: number }} Run */

/**
 * @typedef {{
 *   commit: string,
 *   dirty: boolean,
 *   exeHash: string,
 *   folder: string,
 *   label: string | null,
 *   platform: string,
 *   runs: Run[],
 *   version: string,
 * }} Measured
 */

/** The stages of a reading, in the order the analyzer takes them (spec TIME-2). */
export const STAGES = [
  'handOff',
  'files',
  'marks',
  'packages',
  'tsconfig',
  'scriptsRead',
  'scriptsParse',
  'resolve',
  'entries',
  'build',
];

/** The table's rows, in order: the stages, then what lies around them (spec MEAS-3). */
export const ROWS = [...STAGES, 'print', 'other', 'total', 'start', 'wall'];

/** The kinds of run, in the order the table shows them (spec MEAS-2, MEAS-4). */
const KINDS = ['cold', 'unknown', 'warm', 'first-start'];

/** What the table calls each kind of run. */
const KIND_NAMES = {
  cold: 'cold',
  'first-start': 'first start',
  unknown: 'first run, cache unknown',
  warm: 'warm',
};

/** The fields the documents of one series agree on before their runs are pooled (spec MEAS-3a). */
const AGREED = ['platform', 'exeHash', 'commit', 'dirty', 'label'];

/**
 * Finds the timing line among the lines a run wrote on standard error (spec TIME-5).
 *
 * @param {string} stderr - What the run wrote on standard error.
 * @returns {Timings | undefined} The timings, or nothing when no timing line came.
 */
export const timingLine = (stderr) => {
  for (const line of stderr.split(/\r?\n/u)) {
    if (line.startsWith('{"timings":')) {
      return JSON.parse(line).timings;
    }
  }

  return undefined;
};

/**
 * The kind of the first timed run: cold on the caller's word, or with the file cache emptied first (spec MEAS-4).
 *
 * @param {{ cold: boolean, dropCaches: boolean }} options - The options the script was given.
 * @returns {'cold' | 'unknown'} The kind.
 */
export const coldKind = ({ cold, dropCaches }) => {
  return cold || dropCaches ? 'cold' : 'unknown';
};

/**
 * The middle value, or the mean of the two middle values.
 *
 * @param {readonly number[]} values - The values; at least one.
 * @returns {number} The median.
 */
export const median = (values) => {
  const sorted = [...values].sort((left, right) => {
    return left - right;
  });
  const middle = Math.floor(sorted.length / 2);

  return sorted.length % 2 === 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

/**
 * One run's value for each row of the table, in microseconds. `other` is the time no stage covers, and `start`
 * the time outside the analyzer's own clock: starting the process and ending it (spec TIME-2, MEAS-3).
 *
 * @param {Run} run - The run.
 * @returns {Record<string, number>} Its values, by row.
 */
export const columns = (run) => {
  const { print, reading, total } = run.timings;
  const staged = STAGES.reduce((sum, stage) => {
    return sum + (reading[stage] ?? 0);
  }, 0);

  return {
    ...Object.fromEntries(
      STAGES.map((stage) => {
        return [stage, reading[stage] ?? 0];
      })
    ),
    other: total - staged - print,
    print,
    start: run.wall - total,
    total,
    wall: run.wall,
  };
};

/**
 * The median, lowest and highest of each row over the runs of one kind.
 *
 * @param {readonly Run[]} runs - Every run.
 * @param {string} kind - The kind to sum up.
 * @returns {Record<string, { high: number, low: number, median: number }> | undefined} By row, or nothing when no
 *   run is of that kind.
 */
const summary = (runs, kind) => {
  const chosen = runs
    .filter((run) => {
      return run.kind === kind;
    })
    .map(columns);

  if (chosen.length === 0) {
    return undefined;
  }

  return Object.fromEntries(
    ROWS.map((row) => {
      const values = chosen.map((each) => {
        return each[row];
      });

      return [row, { high: Math.max(...values), low: Math.min(...values), median: median(values) }];
    })
  );
};

/**
 * Pools the runs of documents the script wrote with `--json`. They agree on the platform, the executable, the
 * measured commit, whether it had uncommitted changes, and the label, or nothing is pooled (spec MEAS-3a).
 *
 * @param {readonly Measured[]} documents - The documents.
 * @returns {Measured} One document holding every run.
 * @throws {Error} When there is no document, or two differ.
 */
export const combine = (documents) => {
  const [first] = documents;

  if (first === undefined) {
    throw new Error('nothing to combine: name the files the script wrote with --json');
  }

  for (const document of documents.slice(1)) {
    for (const field of AGREED) {
      if (document[field] !== first[field]) {
        throw new Error(
          `the files differ in ${field} (${String(first[field])} and ${String(document[field])}), so their runs are not pooled`
        );
      }
    }
  }

  return {
    ...first,
    runs: documents.flatMap((document) => {
      return document.runs;
    }),
  };
};

/**
 * A time in milliseconds, to a tenth.
 *
 * @param {number} micros - The time, in microseconds.
 * @returns {string} The time as text.
 */
const ms = (micros) => {
  return (micros / 1000).toFixed(1);
};

/**
 * The table, in Markdown: a row for each stage and for what lies around them, a column for each kind of run that
 * happened and always one for cold runs, and a last column for what a cold run adds to a warm one (spec MEAS-3,
 * MEAS-3a).
 *
 * @param {Measured} document - A measured or combined document.
 * @returns {string} The table, with a line above it saying what was measured.
 */
export const table = (document) => {
  const shown = KINDS.map((kind) => {
    return { kind, summed: summary(document.runs, kind) };
  }).filter(({ kind, summed }) => {
    return summed !== undefined || kind === 'cold';
  });
  const cold = summary(document.runs, 'cold');
  const warm = summary(document.runs, 'warm');
  const head = [
    'stage (ms: median, lowest to highest)',
    ...shown.map(({ kind }) => {
      return KIND_NAMES[kind];
    }),
    'cold minus warm',
  ];
  const changes = document.dirty ? ', with uncommitted changes' : '';
  const label = document.label === null ? '' : `, ${document.label}`;
  const rows = ROWS.map((row) => {
    const cells = shown.map(({ summed }) => {
      if (summed === undefined) {
        return 'none';
      }

      const { high, low, median: middle } = summed[row];

      return `${ms(middle)} (${ms(low)} to ${ms(high)})`;
    });
    const added = cold === undefined || warm === undefined ? 'none' : ms(cold[row].median - warm[row].median);

    return `| ${[row, ...cells, added].join(' | ')} |`;
  });

  return [
    `${document.version} on ${document.platform}, ${document.folder} at ${document.commit}${changes}${label}`,
    '',
    `| ${head.join(' | ')} |`,
    `|${head
      .map(() => {
        return ' --- |';
      })
      .join('')}`,
    ...rows,
  ].join('\n');
};
