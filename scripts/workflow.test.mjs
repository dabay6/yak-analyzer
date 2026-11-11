import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT, TARGETS } from './lib/run.mjs';

/** The workflow's text. It is read as text: nothing here parses YAML, and nothing needs to. */
const workflow = readFileSync(join(ROOT, '.github', 'workflows', 'ci.yml'), 'utf8');

/**
 * Reads a ruleset that is sent to GitHub.
 *
 * @param {string} name - The file's name under `.github/rulesets/`.
 * @returns {Record<string, any>} The ruleset.
 */
const ruleset = (name) => JSON.parse(readFileSync(join(ROOT, '.github', 'rulesets', name), 'utf8'));

/**
 * How many times a piece of text is in the workflow.
 *
 * @param {string} text - The text.
 * @returns {number} The count.
 */
const count = (text) => workflow.split(text).length - 1;

/** The two jobs a pull request runs that are not a target's. */
const NAMED = ['Pack the main package', 'Rehearse the publishing'];

test('the checks required on main are the seven jobs a pull request runs (spec PROT-1, PUB-17)', () => {
  const required = ruleset('main.json').rules[0].parameters.required_status_checks.map(({ context }) => context);
  const built = [...workflow.matchAll(/- \{ target: ([\w-]+), runner: [\w.-]+, image: /gu)].map(([, target]) => target);

  assert.deepEqual(built, TARGETS);
  assert.deepEqual([...required].sort(), [...NAMED, ...TARGETS].sort());

  for (const name of NAMED) {
    assert.match(workflow, new RegExp(`\\n    name: ${name}\\r?\\n`, 'u'), name);
  }
});

test('the published packages are checked on the same five targets (spec PUB-9)', () => {
  const checked = [...workflow.matchAll(/- \{ target: ([\w-]+), runner: [\w.-]+ \}/gu)].map(([, target]) => target);

  assert.deepEqual(checked, TARGETS);
});

test('every action is pinned to a full commit hash, with its version beside it (spec CI-4)', () => {
  const uses = [...workflow.matchAll(/uses: (\S+)(.*)/gu)];

  assert.ok(uses.length > 0);

  for (const [, action, rest] of uses) {
    assert.match(action, /^[\w./-]+@[0-9a-f]{40}$/u, action);
    assert.match(rest.trimEnd(), /^ # v\d+\.\d+\.\d+$/u, action);
  }
});

test("no trigger runs a fork's code with the repository's rights (spec CI-2)", () => {
  assert.equal(count('pull_request_target'), 0);
  assert.equal(count('workflow_run'), 0);
});

test('one job can ask for an identity token, and one other can write the contents (spec CI-3)', () => {
  assert.equal(count('id-token: write'), 1);
  assert.equal(count('contents: write'), 1);
  assert.equal(count('\npermissions:\n  contents: read\n') + count('\r\npermissions:\r\n  contents: read\r\n'), 1);
});

test('the publishing job alone has the marker, the npm token and the publishing environment (spec CI-5, PROT-2)', () => {
  assert.equal(count("YAK_ANALYZER_PUBLISHING_JOB: 'true'"), 1);
  assert.equal(count('secrets.NPM_TOKEN'), 1);
  assert.equal(count('environment: npm'), 1);
  assert.equal(count('--publish'), 1);
});

test("release-please alone has the token that opens the release pull request, in its own environment (spec PUB-1a)", () => {
  assert.equal(count('secrets.RELEASE_PLEASE_TOKEN'), 1);
  assert.equal(count('environment: release'), 1);
});

test('only administrators can make, move or delete a release tag (spec PROT-1a)', () => {
  const tags = ruleset('release-tags.json');

  assert.equal(tags.target, 'tag');
  assert.equal(tags.enforcement, 'active');
  assert.deepEqual(tags.conditions.ref_name.include, ['refs/tags/v*']);
  assert.deepEqual(tags.bypass_actors, [{ actor_id: 5, actor_type: 'RepositoryRole', bypass_mode: 'always' }]);

  for (const rule of ['creation', 'update', 'deletion']) {
    assert.ok(
      tags.rules.some(({ type }) => type === rule),
      rule
    );
  }
});
