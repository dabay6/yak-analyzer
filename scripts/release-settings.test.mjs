import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { test } from 'node:test';

import { MAIN_DIR, manifest, PACKAGE_DIRS, ROOT } from './lib/run.mjs';

/**
 * Reads a file under the repository's root.
 *
 * @param {string} path - The file, from the root.
 * @returns {string} Its text.
 */
const read = (path) => readFileSync(join(ROOT, path), 'utf8');

/** release-please's settings for the one thing it releases, the repository's root. */
const settings = JSON.parse(read('release-please-config.json')).packages['.'];

test('release-please bumps the crate itself, and each of the six manifests as an extra file (spec PKG-8)', () => {
  const manifests = PACKAGE_DIRS.map((dir) => relative(ROOT, join(dir, 'package.json')).replaceAll('\\', '/'));

  assert.equal(settings['release-type'], 'rust');
  assert.deepEqual(
    settings['extra-files'].filter((file) => typeof file === 'string').sort(),
    [...manifests].sort()
  );
});

test("it bumps the main package's optional dependencies too, and nothing else by a path of its own", () => {
  assert.deepEqual(
    settings['extra-files'].filter((file) => typeof file !== 'string'),
    [{ jsonpath: '$.optionalDependencies.*', path: 'npm/yak-analyzer/package.json', type: 'json' }]
  );
});

test('the tag of a release is `v` and the version, with no name before it (spec PUB-2a)', () => {
  assert.equal(settings['include-component-in-tag'], false);
  assert.equal(settings['include-v-in-tag'], true);
});

test("release-please's own record of the version is the version in the tree", () => {
  assert.deepEqual(JSON.parse(read('.release-please-manifest.json')), { '.': manifest(MAIN_DIR).version });
});

test('pull requests move the Cargo pins, the actions and the Rust version (spec PUB-7)', () => {
  const dependabot = read('.github/dependabot.yml');

  for (const ecosystem of ['cargo', 'github-actions', 'rust-toolchain']) {
    assert.ok(dependabot.includes(`package-ecosystem: ${ecosystem}`), ecosystem);
  }
});
