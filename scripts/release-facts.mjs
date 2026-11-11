/**
 * Says whether this commit is a release with something left to publish, from two facts that are still true when
 * a job is run again (spec PUB-2a): a tag for the version in the tree points at this very commit, and that
 * version is not yet whole on npmjs.com. It never asks release-please whether a release was just made.
 *
 * It hands the workflow `tagged` (the tag is on this commit) and `publish` (and a package is still missing).
 */
import { spawnSync } from 'node:child_process';

import { fail, isPublished, MAIN_DIR, manifest, output, PACKAGE_DIRS } from './lib/run.mjs';

/** The version in the tree. */
const { version } = manifest(MAIN_DIR);

/** The tag a release of that version has. */
const tag = `v${version}`;

/**
 * Runs git and gives what it printed.
 *
 * @param {readonly string[]} args - git's arguments.
 * @returns {string} Its standard output, trimmed.
 */
const git = (args) => {
  const ended = spawnSync('git', args, { encoding: 'utf8' });

  if (ended.status !== 0) {
    fail(`git ${args.join(' ')} failed: ${ended.error?.message ?? ended.stderr}`);
  }

  return ended.stdout.trim();
};

/** The commit this run built. */
const commit = process.env.GITHUB_SHA ?? git(['rev-parse', 'HEAD']);

/** The tag as the repository on GitHub has it: its own line, and a second line when the tag is an object. */
const listed = git(['ls-remote', '--tags', 'origin', `refs/tags/${tag}`, `refs/tags/${tag}^{}`])
  .split('\n')
  .filter((line) => {
    return line.length > 0;
  })
  .map((line) => {
    const [sha, ref] = line.split('\t');

    return { ref, sha };
  });

/** The commit the tag points at, through a tag object when there is one. */
const tagged = (listed.find(({ ref }) => ref.endsWith('^{}')) ?? listed[0])?.sha;

/** Whether the tag is on this commit. */
const isTagged = tagged === commit;

/** Whether a package of the version is still missing from npmjs.com. Asked only for a tagged commit. */
const isMissingOne =
  isTagged &&
  PACKAGE_DIRS.some((dir) => {
    return !isPublished(manifest(dir).name, version);
  });

console.log(
  isTagged
    ? `${tag} is on this commit; ${isMissingOne ? 'a package is still to be published' : 'all six packages are published'}`
    : `${tag} is ${tagged === undefined ? 'not a tag yet' : 'on another commit'}; nothing to publish`
);
output('tagged', String(isTagged));
output('publish', String(isMissingOne));
