/**
 * Checks that the release build asks for nothing newer than the Node yak needs (spec PKG-2a): glibc 2.28 on
 * Linux, macOS 11.0, and no C runtime library on Windows. A build takes the libraries of the machine it is made
 * on as its floor unless told otherwise, and that machine's own tests cannot notice.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

import { BUILT, fail } from './lib/run.mjs';

/** The newest glibc the Linux binaries may ask for. */
const GLIBC_FLOOR = '2.28';

/** The newest macOS the macOS binaries may ask for. */
const MACOS_FLOOR = '11.0';

/** The Windows libraries that are a C runtime. */
const C_RUNTIME = /^(?:vcruntime|msvcp|ucrtbase|api-ms-win-crt-)/iu;

/**
 * Runs a program and gives what it printed.
 *
 * @param {string} program - The program.
 * @param {readonly string[]} args - Its arguments.
 * @returns {string} Its standard output.
 */
const printed = (program, args) => {
  const ended = spawnSync(program, args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });

  if (ended.status !== 0) {
    fail(`${program} failed: ${ended.error?.message ?? ended.stderr}`);
  }

  return ended.stdout;
};

/**
 * Whether one dotted version is newer than another.
 *
 * @param {string} version - The version to judge.
 * @param {string} floor - The version to judge it against.
 * @returns {boolean} True when `version` is newer than `floor`.
 */
const newerThan = (version, floor) => {
  const left = version.split('.').map(Number);
  const right = floor.split('.').map(Number);

  for (let i = 0; i < Math.max(left.length, right.length); i += 1) {
    const difference = (left[i] ?? 0) - (right[i] ?? 0);

    if (difference !== 0) {
      return difference > 0;
    }
  }

  return false;
};

/**
 * The newest of some dotted versions.
 *
 * @param {readonly string[]} versions - The versions.
 * @returns {string} The newest.
 */
const newest = (versions) => {
  return versions.reduce((best, version) => {
    return newerThan(version, best) ? version : best;
  });
};

if (!existsSync(BUILT)) {
  fail(`${BUILT} is not built; run \`cargo build --release\` first`);
}

if (process.platform === 'linux') {
  const versions = [...printed('objdump', ['-T', BUILT]).matchAll(/GLIBC_(\d+(?:\.\d+)+)/gu)].map(([, version]) => {
    return version;
  });

  if (versions.length === 0) {
    fail(`objdump shows no glibc version in ${BUILT}`);
  }

  if (newerThan(newest(versions), GLIBC_FLOOR)) {
    fail(`${BUILT} asks for glibc ${newest(versions)}, newer than ${GLIBC_FLOOR}`);
  }

  console.log(`the binary asks for glibc ${newest(versions)} at most`);
} else if (process.platform === 'darwin') {
  const minimum = /LC_BUILD_VERSION[\s\S]*?minos (\S+)/u.exec(printed('otool', ['-l', BUILT]))?.[1];

  if (minimum === undefined) {
    fail(`otool shows no minimum macOS version in ${BUILT}`);
  }

  if (newerThan(minimum, MACOS_FLOOR)) {
    fail(`${BUILT} asks for macOS ${minimum}, newer than ${MACOS_FLOOR}`);
  }

  console.log(`the binary asks for macOS ${minimum} at least`);
} else if (process.platform === 'win32') {
  const vswhere = join(process.env['ProgramFiles(x86)'] ?? '', 'Microsoft Visual Studio', 'Installer', 'vswhere.exe');
  const installation = printed(vswhere, [
    '-latest',
    '-products',
    '*',
    '-requires',
    'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
    '-property',
    'installationPath',
  ]).trim();
  const tools = join(installation, 'VC', 'Tools', 'MSVC');
  const dumpbin = join(tools, readdirSync(tools).sort().at(-1) ?? '', 'bin', 'Hostx64', 'x64', 'dumpbin.exe');
  const libraries = printed(dumpbin, ['/NOLOGO', '/DEPENDENTS', BUILT])
    .split(/\r?\n/u)
    .map((line) => {
      return line.trim();
    })
    .filter((line) => {
      return /^[\w.-]+\.dll$/iu.test(line);
    });
  const runtime = libraries.filter((library) => {
    return C_RUNTIME.test(library);
  });

  if (runtime.length > 0) {
    fail(`${BUILT} needs a C runtime to be installed: ${runtime.join(', ')}`);
  }

  console.log(`the binary carries its C runtime; it imports ${libraries.join(', ')}`);
} else {
  fail(`no floor is known for ${process.platform}`);
}
