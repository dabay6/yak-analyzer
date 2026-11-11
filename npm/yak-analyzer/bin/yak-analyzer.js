#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';

import { launch, linuxLibc } from '#yak-analyzer/launch.js';

/** Resolves a package from where this package is installed, which is where its platform package is too. */
const require = createRequire(import.meta.url);

process.exitCode = launch({
  arch: process.arch,
  args: process.argv.slice(2),
  libc: linuxLibc(),
  platform: process.platform,
  resolve: (id) => {
    return require.resolve(id);
  },
  run: (file, args) => {
    return spawnSync(file, args, { stdio: 'inherit', windowsHide: true });
  },
  write: (line) => {
    process.stderr.write(line);
  },
});
