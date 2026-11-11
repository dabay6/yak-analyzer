/**
 * Publishes the packed files in `dist/`, or tries the publishing without publishing (spec PUB-4, PUB-10, PUB-13,
 * PUB-15 to PUB-17, CI-5). One script does both, so the npm, the registry, the order and the answer for a version
 * that is already published are written once.
 *
 * `node scripts/dry-run-publish.mjs` is the dry run, over whichever packed files `dist/` holds.
 * `--all` asks for all six to be there, as the publishing job has them.
 * `--publish` publishes. It runs only inside the publishing job of the workflow.
 */
import { existsSync } from 'node:fs';

import {
  DIST,
  fail,
  isPublished,
  MAIN_DIR,
  manifest,
  npm,
  output,
  PACKAGE_DIRS,
  packedFile,
  PUBLISHER,
  REGISTRY,
  sleep,
} from './lib/run.mjs';

/** The marker the publishing job sets for this script, and nothing else does. */
const PUBLISHING_JOB = 'YAK_ANALYZER_PUBLISHING_JOB';

/** What npm's dry run says when the version is already published. */
const ALREADY_PUBLISHED = 'cannot publish over the previously published versions';

/** How many times, a quarter of a minute apart, npmjs.com is asked for a package that was just published. */
const TRIES = 20;

/** The arguments this script knows. */
const KNOWN = ['--all', '--publish'];

/** The arguments it was given. */
const given = process.argv.slice(2);

/** Whether to publish for real. */
const publishing = given.includes('--publish');

for (const arg of given) {
  if (!KNOWN.includes(arg)) {
    fail(`unknown argument ${arg}; usage: node scripts/dry-run-publish.mjs [--all] [--publish]`);
  }
}

if (publishing && process.env[PUBLISHING_JOB] !== 'true') {
  fail('only the publishing job of the workflow publishes; run this script without --publish for a dry run');
}

/** The six packages in publishing order: the platform packages, then the main package (spec PUB-4). */
const packages = PACKAGE_DIRS.map((dir) => {
  const { name, version } = manifest(dir);

  return { file: packedFile(dir), isMain: dir === MAIN_DIR, name, version };
});

/** The packages whose packed file is in `dist/`. */
const packed = packages.filter(({ file }) => {
  return existsSync(file);
});

if (packed.length === 0) {
  fail(`${DIST} holds no packed file; run scripts/pack.mjs first`);
}

if ((publishing || given.includes('--all')) && packed.length !== packages.length) {
  const missing = packages.filter(({ file }) => {
    return !existsSync(file);
  });

  fail(`${DIST} must hold all six packed files; missing: ${missing.map(({ name }) => name).join(', ')}`);
}

/**
 * Stops unless every platform package is on npmjs.com at the version, for anyone to install, so that the main
 * package is never published without one (spec PUB-16). A package just published can take a moment to show.
 *
 * @param {string} version - The version being published.
 */
const platformsArePublished = (version) => {
  for (const { isMain, name } of packages) {
    if (isMain) {
      continue;
    }

    let found = false;

    for (let i = 0; i < TRIES && !found; i += 1) {
      if (i > 0) {
        sleep(15);
      }

      found = isPublished(name, version);
    }

    if (!found) {
      fail(`${name}@${version} is not on npmjs.com, so the main package is not published; put it right and run the job again`);
    }
  }
};

/** The packages this run published. */
const published = [];

for (const { file, isMain, name, version } of packed) {
  if (!publishing) {
    const tried = npm('npx', ['--yes', PUBLISHER, 'publish', '--dry-run', '--access', 'public', REGISTRY, file]);
    const said = `${tried.stdout ?? ''}${tried.stderr ?? ''}`;

    if (tried.status === 0) {
      console.log(`the dry run passed for ${name}@${version}`);
    } else if (said.includes(ALREADY_PUBLISHED)) {
      console.log(`the dry run passed for ${name}@${version}; that version is already published`);
    } else {
      console.error(said);
      fail(`npm's dry run refused ${name}@${version}`);
    }

    continue;
  }

  if (isPublished(name, version)) {
    console.log(`${name}@${version} is already published; skipped`);

    continue;
  }

  if (isMain) {
    platformsArePublished(version);
  }

  // Provenance is asked for outright: npm writes it by itself only for a trusted publish (spec PUB-15).
  const sent = npm('npx', ['--yes', PUBLISHER, 'publish', file, '--provenance', '--access', 'public', REGISTRY], {
    stdio: 'inherit',
  });

  if (sent.status !== 0) {
    fail(`npm could not publish ${name}@${version}; put it right and run the job again`);
  }

  published.push(name);
}

if (publishing) {
  const stored = (process.env.NODE_AUTH_TOKEN ?? '') !== '';
  const way = stored
    ? 'A token was stored for this run, so it does not show which way npm took.'
    : 'No token was stored for this run, so they were published by trusted publishing.';

  output('tag', `v${packages[0].version}`);
  output('published', published.join(' '));
  output('line', published.length === 0 ? '' : `Published to npm by one run of the job: ${published.join(', ')}. ${way}`);
}
