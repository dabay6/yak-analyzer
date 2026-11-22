/**
 * Measures how long the analyzer's `graph` takes on a folder, stage by stage: one cold run, ten warm runs and a
 * first start of a fresh copy of the executable (spec MEAS-1 to MEAS-5). Or pools what earlier runs of it wrote
 * with `--json` (spec MEAS-3a).
 *
 *   node scripts/measure.mjs <folder> [--exe <path>] [--hand-off <file>] [--cold | --drop-caches]
 *     [--label <text>] [--json]
 *   node scripts/measure.mjs --combine <file>... [--json]
 *
 * `--cold` says the folder's files are not in the file cache, as on the first run after a restart. On Linux,
 * `--drop-caches` empties the cache first, with passwordless sudo.
 */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { parseArgs } from 'node:util';

import { coldKind, combine, table, timingLine } from './lib/measure.mjs';
import { BINARY, BUILT, fail, TARGET } from './lib/run.mjs';

/** How many warm runs follow the cold one (spec MEAS-2). */
const WARM_RUNS = 10;

/** The variable that turns the analyzer's timings on (spec TIME-1). */
const SWITCH = 'YAK_ANALYZER_TIMINGS';

/** The tiny project the warm-up reads, left untracked, so no commit and no git identity are needed (spec MEAS-2). */
const WARM_UP = {
  'package.json': '{ "name": "warm-up", "main": "src/a.ts" }\n',
  'src/a.ts': "import { b } from './b';\n\nexport const a = b + 1;\n",
  'src/b.ts': "import type { a } from './a';\n\nexport const b = 1;\n",
  'tsconfig.json': '{ "compilerOptions": { "strict": true } }\n',
};

/**
 * Reads the command line, or stops with what was wrong with it.
 *
 * @returns {{ values: Record<string, string | boolean | undefined>, positionals: string[] }} What it holds.
 */
const commandLine = () => {
  try {
    return parseArgs({
      allowPositionals: true,
      options: {
        cold: { default: false, type: 'boolean' },
        combine: { default: false, type: 'boolean' },
        'drop-caches': { default: false, type: 'boolean' },
        exe: { type: 'string' },
        'hand-off': { type: 'string' },
        json: { default: false, type: 'boolean' },
        label: { type: 'string' },
      },
    });
  } catch (error) {
    return fail(`${error.message}; see the head of scripts/measure.mjs for what the script takes`);
  }
};

const { positionals, values: options } = commandLine();

/**
 * Runs a program to its end.
 *
 * @param {string} program - The program.
 * @param {readonly string[]} args - Its arguments.
 * @param {import('node:child_process').SpawnSyncOptions} [settings] - Where to run it, and what it gets.
 * @returns {import('node:child_process').SpawnSyncReturns<string>} How it ended.
 */
const ran = (program, args, settings = {}) => {
  return spawnSync(program, args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, ...settings });
};

/**
 * The first line of what a program that failed wrote, or why it could not start.
 *
 * @param {import('node:child_process').SpawnSyncReturns<string>} ended - How it ended.
 * @returns {string} The line.
 */
const firstLine = (ended) => {
  return (ended.error?.message ?? ended.stderr ?? '').split(/\r?\n/u)[0] || `it ended with exit code ${ended.status}`;
};

/**
 * Prints a measured or combined document: as a table, or with `--json` as one JSON document (spec MEAS-3).
 *
 * @param {import('./lib/measure.mjs').Measured} document - The document.
 */
const show = (document) => {
  console.log(options.json ? JSON.stringify(document) : table(document));
};

/** Pools the files `--combine` names, and prints them (spec MEAS-3a). */
const pooled = () => {
  if (positionals.length === 0) {
    fail('--combine needs the files the script wrote with --json');
  }

  try {
    show(
      combine(
        positionals.map((file) => {
          return JSON.parse(readFileSync(file, 'utf8'));
        })
      )
    );
  } catch (error) {
    fail(error.message);
  }
};

/** Empties the file cache, on Linux, through passwordless sudo, with the script itself an ordinary user (spec MEAS-4). */
const dropCaches = () => {
  if (process.platform !== 'linux') {
    fail('--drop-caches works on Linux only; elsewhere run the script first after a restart, and say so with --cold');
  }

  const dropped = ran('sudo', ['-n', 'sh', '-c', 'sync; echo 3 > /proc/sys/vm/drop_caches']);

  if (dropped.status !== 0) {
    fail(`the file cache could not be emptied, which needs passwordless sudo: ${firstLine(dropped)}`);
  }
};

/**
 * Runs git in a folder, without taking a lock, and gives what it printed.
 *
 * @param {string} folder - The folder.
 * @param {readonly string[]} args - Git's arguments.
 * @returns {string} What it printed, trimmed.
 */
const git = (folder, args) => {
  const ended = ran('git', ['--no-optional-locks', ...args], { cwd: folder });

  if (ended.status !== 0) {
    fail(`git ${args.join(' ')} failed in ${folder}: ${firstLine(ended)}`);
  }

  return ended.stdout.trim();
};

/**
 * Runs `graph` on the warm-up project, untimed, which starts the executable for the first time and loads nearly
 * all the analyzer's and git's code from disk before the cold run (spec MEAS-2).
 *
 * @param {string} exe - The executable.
 * @param {string} temporary - The script's temporary folder.
 * @param {string} fix - What to do when it does not run.
 */
const warmUp = (exe, temporary, fix) => {
  const project = join(temporary, 'warm-up');

  for (const [path, text] of Object.entries(WARM_UP)) {
    mkdirSync(dirname(join(project, path)), { recursive: true });
    writeFileSync(join(project, path), text);
  }

  git(project, ['init', '--quiet']);

  const ended = ran(exe, ['graph'], { cwd: project, env: { ...process.env, [SWITCH]: '' }, stdio: ['ignore', 'ignore', 'pipe'] });

  if (ended.status !== 0) {
    fail(`${exe} does not run: ${firstLine(ended)}${fix}`);
  }
};

/**
 * Runs `graph` once, timed, and records it. A run that fails, or writes no timing line, stops the script, so a
 * broken run is never averaged in (spec MEAS-5).
 *
 * @param {string} exe - The executable.
 * @param {string} folder - The folder measured.
 * @param {readonly string[]} args - The arguments of `graph`.
 * @param {string} kind - The kind of run.
 * @returns {import('./lib/measure.mjs').Run} The run.
 */
const timed = (exe, folder, args, kind) => {
  const started = process.hrtime.bigint();
  const ended = ran(exe, args, { cwd: folder, env: { ...process.env, [SWITCH]: '1' }, stdio: ['ignore', 'ignore', 'pipe'] });
  const wall = Number((process.hrtime.bigint() - started) / 1000n);

  if (ended.status !== 0) {
    fail(firstLine(ended));
  }

  const timings = timingLine(ended.stderr);

  if (timings === undefined) {
    fail(`${exe} wrote no timing line; it may be older than the timing switch, so build it again`);
  }

  return { kind, timings, wall };
};

/** Measures the folder the command line names, and prints what it measured (spec MEAS-1 to MEAS-4). */
const measure = () => {
  if (positionals.length !== 1) {
    fail('name the one folder to measure: node scripts/measure.mjs <folder>');
  }

  const folder = resolve(positionals[0]);
  const exe = resolve(options.exe ?? BUILT);
  const fix = options.exe === undefined ? '; build it with `cargo build --release`' : '';
  const args = ['graph', ...(options['hand-off'] === undefined ? [] : ['--hand-off', resolve(options['hand-off'])])];

  if (!existsSync(exe)) {
    fail(`there is no executable at ${exe}${fix}`);
  }

  if (options['drop-caches']) {
    dropCaches();
  }

  const temporary = mkdtempSync(join(tmpdir(), 'yak-measure-'));

  // The temporary folder goes when the script ends, also when it stops on a failure or is interrupted.
  process.on('exit', () => {
    rmSync(temporary, { force: true, recursive: true });
  });
  process.on('SIGINT', () => {
    process.exit(130);
  });

  warmUp(exe, temporary, fix);

  const runs = [timed(exe, folder, args, coldKind({ cold: options.cold, dropCaches: options['drop-caches'] }))];

  for (let i = 0; i < WARM_RUNS; i += 1) {
    runs.push(timed(exe, folder, args, 'warm'));
  }

  const copy = join(temporary, 'copy', BINARY);

  mkdirSync(dirname(copy));
  copyFileSync(exe, copy);
  runs.push(timed(copy, folder, args, 'first-start'));

  // Read only now, after the cold run, so that git does not warm the folder's index first (spec MEAS-2).
  show({
    commit: git(folder, ['rev-parse', 'HEAD']),
    dirty: git(folder, ['status', '--porcelain']) !== '',
    exeHash: createHash('sha256').update(readFileSync(exe)).digest('hex'),
    folder,
    label: options.label ?? null,
    platform: TARGET,
    runs,
    version: ran(exe, ['--version']).stdout.trim(),
  });
};

if (options.combine) {
  pooled();
} else {
  measure();
}
