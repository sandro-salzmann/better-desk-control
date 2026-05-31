#!/usr/bin/env node
// Bump every crate to one shared version, keep their Cargo.lock entries in sync,
// then commit and tag `v<version>`. The release workflow triggers on `v*`, so
// the tag must track this version.
//
//   node scripts/bump.mjs                  # CalVer from today's date
//
// Versions are CalVer (https://calver.org), not semver: `YYYY.MM.MICRO`, where
// MICRO is a within-month counter (0, 1, ...) so several releases in one month
// stay ordered. The updater needs a valid semver string and compares the dotted
// numbers, which a date in this order satisfies: a later month or year always
// sorts higher. Semver forbids leading zeros, so the month is NOT zero-padded.

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// JSON files carry the version in a top-level "version" key; Cargo.toml files in
// their [package] block. Every crate name also appears once in the shared lock.
const JSON_FILES = ["package.json", "src-tauri/tauri.conf.json"];
const TOML_FILES = [
  "src-tauri/Cargo.toml",
  "desk-cli/Cargo.toml",
  "crates/desk-core/Cargo.toml",
];
const LOCK_NAMES = ["better-desk-control", "desk-cli", "desk-core"];

const current = JSON.parse(
  readFileSync(join(root, "package.json"), "utf8"),
).version;
const [maj, min, pat] = current.split(".").map(Number);

// Derive the version from today. MICRO continues this month's run or resets to
// 0 when the month (or year) has rolled over.
const now = new Date();
const year = now.getFullYear();
const month = now.getMonth() + 1;
const micro = maj === year && min === month ? pat + 1 : 0;
const next = `${year}.${month}.${micro}`;

/** Replace the first match of `regex` in `path`, asserting it exists. */
function patch(path, regex, replacement) {
  const full = join(root, path);
  const text = readFileSync(full, "utf8");
  if (!regex.test(text)) {
    console.error(`could not find version field in ${path}`);
    process.exit(1);
  }
  writeFileSync(full, text.replace(regex, replacement));
  console.log(`  ${path}`);
}

console.log(`Bumping all crates ${current} -> ${next}`);

// JSON: the sole top-level "version" key (dependency entries key on package name).
for (const file of JSON_FILES)
  patch(file, /"version":\s*"[^"]*"/, `"version": "${next}"`);
// Cargo.toml: the version line inside [package] (scoped past any dep versions).
for (const file of TOML_FILES)
  patch(file, /(\[package\][\s\S]*?\nversion = ")[^"]*(")/, `$1${next}$2`);
// Cargo.lock: cargo writes `version` immediately after each crate's `name`.
for (const name of LOCK_NAMES)
  patch(
    "Cargo.lock",
    new RegExp(`(name = "${name}"\\nversion = ")[^"]*(")`),
    `$1${next}$2`,
  );

const tag = `v${next}`;
const git = (...a) => execFileSync("git", a, { cwd: root, stdio: "inherit" });
git("commit", "-am", `chore: release ${tag}`);
git("tag", tag);
console.log(`\nTagged ${tag}. Push it with:\n  git push --follow-tags`);
