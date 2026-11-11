/**
 * Tries the publishing on every packed file in `dist/`, without publishing (spec PUB-10), with the npm that will
 * publish. Between releases the version in the tree is one already on npmjs.com, and npm's dry run refuses a
 * version that exists; that one refusal counts as a pass.
 */
import { existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

import { DIST, fail, npm } from './lib/run.mjs';

/** The npm that publishes. Trusted publishing needs 11.5.1 or later, and npm 12 does not run on Node 22.18. */
const PUBLISHER = 'npm@11.21.2';

/** Where the packages are published, said on the command so that a machine's own settings cannot change it. */
const REGISTRY = '--@dabay6:registry=https://registry.npmjs.org/';

/** What npm says when the version is already published. */
const ALREADY_PUBLISHED = 'cannot publish over the previously published versions';

/** The packed files. */
const files = existsSync(DIST)
  ? readdirSync(DIST)
      .filter((file) => {
        return file.endsWith('.tgz');
      })
      .sort()
  : [];

if (files.length === 0) {
  fail(`${DIST} holds no packed file; run scripts/pack.mjs first`);
}

for (const file of files) {
  const tried = npm('npx', ['--yes', PUBLISHER, 'publish', '--dry-run', '--access', 'public', REGISTRY, join(DIST, file)]);
  const said = `${tried.stdout ?? ''}${tried.stderr ?? ''}`;

  if (tried.status === 0) {
    console.log(`the dry run passed for ${file}`);
  } else if (said.includes(ALREADY_PUBLISHED)) {
    console.log(`the dry run passed for ${file}; its version is already published`);
  } else {
    console.error(said);
    fail(`npm's dry run refused ${file}`);
  }
}
