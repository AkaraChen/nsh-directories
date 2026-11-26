#!/usr/bin/env node
// Verifies or updates the pinned upstream `directories` metadata across:
//   - upstream.json
//   - Cargo.toml
//   - Cargo.lock
//   - src/provenance.rs
//   - src/lib.rs (public API surface)
//   - NOTICE
//
// Usage:
//   node scripts/sync-upstream.mjs --check [upstream-dir]
//   node scripts/sync-upstream.mjs --check-latest
//   node scripts/sync-upstream.mjs --update <version> <commit>

import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function fail(message) {
  console.error(`error: ${message}`);
  process.exit(1);
}

function readText(relPath) {
  return readFileSync(join(root, relPath), "utf8");
}

function extractRustImplMethods(source, structName) {
  const implMarker = `impl ${structName} {`;
  const start = source.indexOf(implMarker);
  if (start === -1) {
    fail(`could not find \`${implMarker}\` in Rust source`);
  }
  let depth = 0;
  let bodyStart = -1;
  let bodyEnd = -1;
  for (let i = start; i < source.length; i++) {
    const ch = source[i];
    if (ch === "{") {
      if (depth === 0) bodyStart = i + 1;
      depth++;
    } else if (ch === "}") {
      depth--;
      if (depth === 0) {
        bodyEnd = i;
        break;
      }
    }
  }
  if (bodyStart === -1 || bodyEnd === -1) {
    fail(`unterminated impl block for ${structName}`);
  }
  const block = source.slice(bodyStart, bodyEnd);
  const methods = [];
  for (const match of block.matchAll(/\bpub\s+(?:const\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(/g)) {
    methods.push(match[1]);
  }
  return methods;
}

function parseCargoLockPackages(lockText) {
  const packages = new Map();
  for (const block of lockText.split(/\n\[\[package\]\]\n/)) {
    const nameMatch = block.match(/^name = "([^"]+)"/m);
    const versionMatch = block.match(/^version = "([^"]+)"/m);
    if (nameMatch && versionMatch) {
      const list = packages.get(nameMatch[1]) ?? [];
      list.push(versionMatch[1]);
      packages.set(nameMatch[1], list);
    }
  }
  return packages;
}

function parseUpstreamMetadata(upstreamDir) {
  const cargoToml = readFileSync(join(upstreamDir, "Cargo.toml"), "utf8");
  const versionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
  const dirsSysMatch = cargoToml.match(/^dirs-sys\s*=\s*"([^"]+)"/m);
  if (!versionMatch || !dirsSysMatch) {
    fail(`could not parse version / dirs-sys from ${join(upstreamDir, "Cargo.toml")}`);
  }
  const libRs = readFileSync(join(upstreamDir, "src/lib.rs"), "utf8");
  const apiSurface = {
    BaseDirs: extractRustImplMethods(libRs, "BaseDirs"),
    UserDirs: extractRustImplMethods(libRs, "UserDirs"),
    ProjectDirs: extractRustImplMethods(libRs, "ProjectDirs"),
  };
  let gitHead = null;
  if (existsSync(join(upstreamDir, ".git"))) {
    gitHead = execFileSync("git", ["-C", upstreamDir, "rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  }
  return {
    version: versionMatch[1],
    dirsSysVersion: dirsSysMatch[1],
    apiSurface,
    gitHead,
  };
}

function check(upstreamArg) {
  const manifest = JSON.parse(readText("upstream.json"));
  const cargoToml = readText("Cargo.toml");
  const provenanceRs = readText("src/provenance.rs");
  const libRs = readText("src/lib.rs");
  const notice = readText("NOTICE");

  const cargoDep = cargoToml.match(/^directories\s*=\s*"([^"]+)"/m);
  if (!cargoDep) {
    fail("missing `directories = \"...\"` dependency in Cargo.toml");
  }
  if (cargoDep[1] !== manifest.version && cargoDep[1] !== `=${manifest.version}`) {
    fail(`Cargo.toml pins directories = ${JSON.stringify(cargoDep[1])}, expected ${JSON.stringify(manifest.version)}`);
  }

  if (!existsSync(join(root, "Cargo.lock"))) {
    fail("Cargo.lock is missing; run `cargo generate-lockfile` and commit Cargo.lock");
  }
  const lockPackages = parseCargoLockPackages(readText("Cargo.lock"));
  const lockedDirs = lockPackages.get(manifest.crate);
  if (!lockedDirs || lockedDirs.length !== 1 || lockedDirs[0] !== manifest.version) {
    fail(`Cargo.lock has ${manifest.crate} = ${JSON.stringify(lockedDirs ?? [])}, expected [${JSON.stringify(manifest.version)}]`);
  }
  const lockedSys = lockPackages.get("dirs-sys");
  if (!lockedSys || lockedSys.length !== 1 || lockedSys[0] !== manifest.dirsSysVersion) {
    fail(`Cargo.lock has dirs-sys = ${JSON.stringify(lockedSys ?? [])}, expected [${JSON.stringify(manifest.dirsSysVersion)}]`);
  }

  const checkConst = (name, expected) => {
    const re = new RegExp(`pub const ${name}:\\s*&str\\s*=\\s*"([^"]+)";`);
    const match = provenanceRs.match(re);
    if (!match || match[1] !== expected) {
      fail(`src/provenance.rs ${name} is ${JSON.stringify(match?.[1] ?? null)}, expected ${JSON.stringify(expected)}`);
    }
  };
  checkConst("UPSTREAM_REPO", manifest.repository);
  checkConst("UPSTREAM_CRATE", manifest.crate);
  checkConst("UPSTREAM_VERSION", manifest.version);
  checkConst("UPSTREAM_COMMIT", manifest.commit);
  checkConst("UPSTREAM_DIRS_SYS_VERSION", manifest.dirsSysVersion);

  for (const [structName, expectedMethods] of Object.entries(manifest.apiSurface)) {
    const actual = extractRustImplMethods(libRs, structName);
    if (JSON.stringify(actual) !== JSON.stringify(expectedMethods)) {
      fail(
        `src/lib.rs ${structName} methods ${JSON.stringify(actual)} differ from upstream.json ${JSON.stringify(expectedMethods)}`,
      );
    }
  }

  if (!notice.includes(manifest.version) || !notice.includes(manifest.commit)) {
    fail(`NOTICE must mention pinned version ${manifest.version} and commit ${manifest.commit}`);
  }

  const candidateDir = upstreamArg ? resolve(upstreamArg) : join(root, "upstream");
  if (existsSync(join(candidateDir, "src/lib.rs"))) {
    const upstream = parseUpstreamMetadata(candidateDir);
    if (upstream.version !== manifest.version) {
      fail(`upstream checkout version is ${upstream.version}, expected ${manifest.version}`);
    }
    if (upstream.dirsSysVersion !== manifest.dirsSysVersion) {
      fail(`upstream dirs-sys version is ${upstream.dirsSysVersion}, expected ${manifest.dirsSysVersion}`);
    }
    if (upstream.gitHead && upstream.gitHead !== manifest.commit) {
      fail(`upstream checkout HEAD is ${upstream.gitHead}, expected ${manifest.commit}`);
    }
    if (JSON.stringify(upstream.apiSurface) !== JSON.stringify(manifest.apiSurface)) {
      fail(`upstream checkout API surface differs from upstream.json`);
    }
    console.log(`upstream checkout verified at ${candidateDir}`);
  }

  console.log(
    `ok: nsh-directories matches ${manifest.crate} ${manifest.version} (dirs-sys ${manifest.dirsSysVersion}, commit ${manifest.commit})`,
  );
}

async function fetchJson(url) {
  const res = await fetch(url, {
    headers: {
      "User-Agent": "nsh-directories-sync/0.1.0 (+https://github.com/AkaraChen/nsh-directories)",
      Accept: "application/json",
    },
  });
  if (!res.ok) {
    throw new Error(`GET ${url} failed with HTTP ${res.status}`);
  }
  return res.json();
}

async function checkLatest() {
  const manifest = JSON.parse(readText("upstream.json"));
  const [branchInfo, crateInfo] = await Promise.all([
    fetchJson("https://codeberg.org/api/v1/repos/dirs/directories-rs/branches/main"),
    fetchJson("https://crates.io/api/v1/crates/directories"),
  ]);
  const latestCommit = branchInfo?.commit?.id;
  const latestVersion = crateInfo?.crate?.max_stable_version ?? crateInfo?.crate?.newest_version;
  if (!latestCommit || !latestVersion) {
    fail("unexpected response from Codeberg or crates.io");
  }

  const diffs = [];
  if (latestVersion !== manifest.version) {
    diffs.push(`crates.io version: pinned=${manifest.version}, latest=${latestVersion}`);
  }
  if (latestCommit !== manifest.commit) {
    diffs.push(`Codeberg main commit: pinned=${manifest.commit}, latest=${latestCommit}`);
  }
  if (diffs.length > 0) {
    console.error("upstream drift detected:\n  - " + diffs.join("\n  - "));
    process.exit(1);
  }
  console.log(`up to date: ${manifest.crate} ${manifest.version} @ ${manifest.commit}`);
}

function renderProvenance(manifest) {
  return `//! Provenance metadata for the pinned upstream \`directories\` crate.
//!
//! Maintained by \`scripts/sync-upstream.mjs\`.

/// Upstream source repository URL.
pub const UPSTREAM_REPO: &str = "${manifest.repository}";

/// Upstream crates.io package name.
pub const UPSTREAM_CRATE: &str = "${manifest.crate}";

/// Pinned upstream \`directories\` crate version.
pub const UPSTREAM_VERSION: &str = "${manifest.version}";

/// Pinned upstream repository commit SHA.
pub const UPSTREAM_COMMIT: &str = "${manifest.commit}";

/// Pinned upstream \`dirs-sys\` version used by \`directories\`.
pub const UPSTREAM_DIRS_SYS_VERSION: &str = "${manifest.dirsSysVersion}";
`;
}

function update(version, commit) {
  if (!version || !commit || !/^[0-9a-f]{40}$/i.test(commit)) {
    console.error("usage: node scripts/sync-upstream.mjs --update <version> <40-char-commit-sha>");
    process.exit(2);
  }
  const manifest = JSON.parse(readText("upstream.json"));
  const tempRoot = mkdtempSync(join(tmpdir(), "nsh-directories-upstream-"));
  try {
    const repoUrl = `${manifest.repository}.git`;
    execFileSync("git", ["clone", "--no-checkout", repoUrl, tempRoot], { stdio: "inherit" });
    execFileSync("git", ["-C", tempRoot, "checkout", "--detach", commit], { stdio: "inherit" });

    const upstream = parseUpstreamMetadata(tempRoot);
    if (upstream.version !== version) {
      fail(`upstream Cargo.toml at ${commit} is version ${upstream.version}, expected ${version}`);
    }

    const updatedManifest = {
      ...manifest,
      version: upstream.version,
      commit,
      dirsSysVersion: upstream.dirsSysVersion,
      apiSurface: upstream.apiSurface,
    };
    writeFileSync(join(root, "upstream.json"), JSON.stringify(updatedManifest, null, 2) + "\n");

    const cargoToml = readText("Cargo.toml").replace(
      /^directories\s*=\s*"[^"]+"/m,
      `directories = "${upstream.version}"`,
    );
    writeFileSync(join(root, "Cargo.toml"), cargoToml);
    writeFileSync(join(root, "src/provenance.rs"), renderProvenance(updatedManifest));

    const notice = readText("NOTICE")
      .replace(/version [^\s,]+,\s*commit [0-9a-f]{40}/i, `version ${upstream.version}, commit ${commit}`)
      .replace(/\(`dirs-sys` [^)]+\)/, `(\`dirs-sys\` ${upstream.dirsSysVersion})`);
    writeFileSync(join(root, "NOTICE"), notice);

    console.log(`updated pin to ${upstream.version} (${commit}); run \`cargo update -p directories\` and \`node scripts/sync-upstream.mjs --check\``);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

const [mode, ...args] = process.argv.slice(2);
if (mode === "--check") {
  check(args[0]);
} else if (mode === "--check-latest") {
  await checkLatest();
} else if (mode === "--update") {
  update(args[0], args[1]);
} else {
  console.error("usage: node scripts/sync-upstream.mjs <--check [dir] | --check-latest | --update <version> <commit>>");
  process.exit(2);
}
