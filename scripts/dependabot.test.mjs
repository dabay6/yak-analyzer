import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** A text file of the repository, with line feeds whatever the checkout wrote. */
const read = (name) => readFileSync(join(ROOT, name), 'utf8').replaceAll('\r\n', '\n');

/** The resolver is released on its own, versioned apart from the other oxc crates, and needs none of them. */
const ALONE = 'oxc_resolver';

test('the oxc crates are pinned to one version, since they are released together (spec AN-2, DEP-3)', () => {
  const pins = [...read('Cargo.toml').matchAll(/^(?<crate>oxc_[a-z_]+) = "=(?<version>[^"]+)"$/gmu)].filter(
    (pin) => pin.groups.crate !== ALONE,
  );

  assert.ok(pins.length > 0);
  assert.equal(new Set(pins.map((pin) => pin.groups.version)).size, 1);
});

test('Dependabot moves the oxc crates together, in one pull request, and the resolver alone (spec DEP-3)', () => {
  const dependabot = read('.github/dependabot.yml');

  assert.match(
    dependabot,
    /- package-ecosystem: cargo\n(?: {4}.*\n)* {4}groups:\n {6}oxc:\n {8}patterns:\n {10}- "oxc_\*"\n {8}exclude-patterns:\n {10}- "oxc_resolver"\n/u,
  );
});
