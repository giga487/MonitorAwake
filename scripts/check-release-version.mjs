import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const json = (path) =>
  JSON.parse(readFileSync(new URL(`../${path}`, import.meta.url), 'utf8'));
const config = json('src-tauri/tauri.conf.json');
const version = config.version;
assert.match(
  version,
  /^\d+\.\d+\.\d+$/,
  'Use a stable x.y.z application version.',
);
assert.equal(
  process.env.RELEASE_TAG,
  `v${version}`,
  'Tag must match the application version.',
);
assert.equal(
  json('package.json').version,
  version,
  'package.json version differs.',
);
const lock = json('package-lock.json');
assert.equal(lock.version, version, 'package-lock.json version differs.');
assert.equal(
  lock.packages[''].version,
  version,
  'npm root package version differs.',
);
const cargo = readFileSync(
  new URL('../src-tauri/Cargo.toml', import.meta.url),
  'utf8',
);
assert.equal(
  cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1],
  version,
  'Cargo.toml version differs.',
);
const cargoLock = readFileSync(
  new URL('../Cargo.lock', import.meta.url),
  'utf8',
);
assert.equal(
  cargoLock.match(
    /\[\[package\]\]\s+name = "monitorawake"\s+version = "([^"]+)"/,
  )?.[1],
  version,
  'Cargo.lock version differs.',
);
console.log(
  `Release ${process.env.RELEASE_TAG}: all application versions agree.`,
);
