/**
 * Packs one package into `dist/`, as the file npm would publish (spec PKG-9).
 *
 * `node scripts/pack.mjs main` packs the main package. `node scripts/pack.mjs platform` copies the release build
 * of the binary into this machine's platform package, and packs that.
 */
import { chmodSync, copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

import { BINARY, BUILT, DIST, fail, MAIN_DIR, npm, PLATFORM_DIR, ROOT, TARGET } from './lib/run.mjs';

/** Which package to pack. */
const which = process.argv[2];

if (which !== 'main' && which !== 'platform') {
  fail('usage: node scripts/pack.mjs main|platform');
}

/** The folder of the package to pack. */
const dir = which === 'main' ? MAIN_DIR : PLATFORM_DIR;

if (!existsSync(dir)) {
  fail(`there is no package for ${TARGET}: ${dir} does not exist`);
}

if (which === 'platform') {
  if (!existsSync(BUILT)) {
    fail(`${BUILT} is not built; run \`cargo build --release\` first`);
  }

  copyFileSync(BUILT, join(dir, BINARY));
  chmodSync(join(dir, BINARY), 0o755);
}

copyFileSync(join(ROOT, 'LICENSE'), join(dir, 'LICENSE'));
mkdirSync(DIST, { recursive: true });

/** How `npm pack` ended. */
const packed = npm('npm', ['pack', '--pack-destination', DIST], { cwd: dir, stdio: 'inherit' });

if (packed.status !== 0) {
  fail(`npm pack failed in ${dir}`);
}
