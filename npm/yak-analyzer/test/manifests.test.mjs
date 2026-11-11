import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { TARGETS } from '#yak-analyzer/launch.js';

/** The repository's root. */
const ROOT = join(import.meta.dirname, '..', '..', '..');

/**
 * Reads a JSON file under the repository's root.
 *
 * @param {...string} path - The file's path, in parts.
 * @returns {Record<string, any>} The parsed file.
 */
const json = (...path) => JSON.parse(readFileSync(join(ROOT, ...path), 'utf8'));

/**
 * The version a Cargo file gives the crate.
 *
 * @param {string} file - `Cargo.toml` or `Cargo.lock`.
 * @returns {string | undefined} The version on the line after the crate's name.
 */
const cargoVersion = (file) =>
  /name = "yak-analyzer"\r?\nversion = "([^"]+)"/u.exec(readFileSync(join(ROOT, file), 'utf8'))?.[1];

const main = json('npm', 'yak-analyzer', 'package.json');
const platforms = TARGETS.map((target) => ({ manifest: json('npm', 'platforms', target, 'package.json'), target }));
const packages = [main, ...platforms.map(({ manifest }) => manifest)];

test('every place the version is written agrees (spec PKG-8)', () => {
  const version = cargoVersion('Cargo.toml');

  assert.match(version, /^\d+\.\d+\.\d+$/u);
  assert.equal(cargoVersion('Cargo.lock'), version);

  for (const manifest of packages) {
    assert.equal(manifest.version, version, manifest.name);
  }

  for (const [name, range] of Object.entries(main.optionalDependencies)) {
    assert.equal(range, version, name);
  }
});

test("the main package's optional dependencies are exactly the five platform packages (spec PKG-3)", () => {
  assert.deepEqual(
    Object.keys(main.optionalDependencies).sort(),
    TARGETS.map((target) => `@dabay6/yak-analyzer-${target}`).sort()
  );
});

test('each platform package is named for its target and states its system, processor and C library', () => {
  for (const { manifest, target } of platforms) {
    const [os, cpu] = target.split('-');

    assert.equal(manifest.name, `@dabay6/yak-analyzer-${target}`);
    assert.deepEqual(manifest.os, [os], target);
    assert.deepEqual(manifest.cpu, [cpu], target);
    assert.deepEqual(manifest.libc, os === 'linux' ? ['glibc'] : undefined, target);
    assert.deepEqual(manifest.files, [os === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer'], target);
  }
});

test('the main package names the launcher as its bin, and the oldest Node it runs on (spec PKG-1, PKG-5)', () => {
  assert.equal(main.name, '@dabay6/yak-analyzer');
  assert.deepEqual(main.bin, { 'yak-analyzer': 'bin/yak-analyzer.js' });
  assert.deepEqual(main.engines, { node: '>=22.18.0' });
});

test('no package has a script of any kind (spec PKG-4)', () => {
  for (const manifest of packages) {
    assert.equal(manifest.scripts, undefined, manifest.name);
  }
});

test('every package carries its metadata (spec PKG-10)', () => {
  for (const manifest of packages) {
    assert.ok(manifest.description.length > 0, manifest.name);
    assert.ok(manifest.keywords.length > 0, manifest.name);
    assert.equal(manifest.license, 'MIT', manifest.name);
    assert.equal(manifest.homepage, 'https://github.com/dabay6/yak-analyzer#readme', manifest.name);
    assert.deepEqual(manifest.bugs, { url: 'https://github.com/dabay6/yak-analyzer/issues' }, manifest.name);
    assert.equal(manifest.repository.url, 'git+https://github.com/dabay6/yak-analyzer.git', manifest.name);
    assert.deepEqual(manifest.publishConfig, { access: 'public' }, manifest.name);
  }
});
