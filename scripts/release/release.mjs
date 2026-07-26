#!/usr/bin/env node
// Local release pipeline for CodeBerth.
//
// Commands:
//   check    verify tauri.conf.json / package.json / Cargo.toml all share the
//            same version
//   build    run `pnpm build:desktop` to produce the NSIS installer
//   sign     sign the installer with the private key (writes the .sig file)
//   latest   write latest.json next to the installer, ready for upload
//   all      check → build → sign → latest (default)
//
// Flags:
//   --skip-build    skip the build step (sign + latest on existing artifacts)
//   --key <path>    override the private key file path
//                   (default: ~/.tauri/project-manger.key)
//   --repo <slug>   override the GitHub repo for the latest.json URL
//                   (default: read from `git remote get-url github`)
//   --version <v>   override the version (debug only; defaults to tauri.conf.json)
//
// IMPORTANT — Tauri 2.11.4 signer CLI bug:
// When TAURI_SIGNING_PRIVATE_KEY_PASSWORD is *undefined*, the signer silently
// fails on a no-password key ("Signing without password." then exit 0 with no
// .sig written). Setting it to an empty string forces the no-password branch
// to actually run. The `sign` command below does this unconditionally.

import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { homedir } from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, "..", "..");

const TAURI_CONF = join(REPO_ROOT, "src-tauri", "tauri.conf.json");
const PACKAGE_JSON = join(REPO_ROOT, "package.json");
const CARGO_TOML = join(REPO_ROOT, "src-tauri", "Cargo.toml");
const BUNDLE_DIR = join(REPO_ROOT, "src-tauri", "target", "release", "bundle", "nsis");

const USAGE = `Usage: node scripts/release/release.mjs <command> [flags]

Commands:
  check    verify three-way version consistency
  build    run pnpm build:desktop
  sign     sign the installer
  latest   write latest.json
  all      check → build → sign → latest (default)

Flags:
  --skip-build    skip build, sign+latest on existing artifacts
  --key <path>    private key path (default: ~/.tauri/project-manger.key)
  --repo <slug>   GitHub repo for the latest.json URL
                  (default: read from \`git remote get-url github\`)
  --version <v>   override version (default: from tauri.conf.json)
  -h, --help      show this help`;

function parseArgs(argv) {
  const args = argv.slice(2);
  const opts = {
    command: "all",
    skipBuild: false,
    // null = "auto-discover from ~/.tauri/" when needed; string = explicit
    // override (set by --key).
    key: null,
    repo: null,
    version: null,
  };
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    switch (a) {
      case "-h":
      case "--help":
        console.log(USAGE);
        process.exit(0);
      case "--skip-build":
        opts.skipBuild = true;
        break;
      case "--key":
        opts.key = resolve(args[++i]);
        break;
      case "--repo":
        opts.repo = args[++i];
        break;
      case "--version":
        opts.version = args[++i];
        break;
      default:
        if (!a.startsWith("-") && !opts.commandSet) {
          opts.command = a;
          opts.commandSet = true;
        } else {
          console.error(`Unknown arg: ${a}`);
          console.log(USAGE);
          process.exit(2);
        }
    }
  }
  return opts;
}

function readVersion(file) {
  // Match the first semver-looking `version = "x.y.z"` or `"version": "x.y.z"`
  // declaration. For Cargo.toml we anchor to start-of-line so we don't pick
  // up unrelated `version = "1"` entries in [dependencies]. JSON files are
  // usually pretty-printed with leading whitespace, so we don't anchor.
  const text = readFileSync(file, "utf8");
  const re = file.endsWith(".toml")
    ? /^version\s*=\s*["']([^"']+)["']/m
    : /["']?version["']?\s*[:=]\s*["']([^"']+)["']/;
  const m = text.match(re);
  if (!m) throw new Error(`Cannot find version in ${file}`);
  return m[1];
}

function readTauriConf() {
  return JSON.parse(readFileSync(TAURI_CONF, "utf8"));
}

const TAURI_KEY_DIR = join(homedir(), ".tauri");

// Auto-discover the signing keypair under ~/.tauri/. Returns the private key
// path. Throws if zero or multiple .key files are present, or if the matching
// .pub file's pubkey does not match tauri.conf.json.
function discoverKey() {
  let candidates;
  try {
    candidates = readdirSync(TAURI_KEY_DIR)
      .filter((f) => f.endsWith(".key"))
      .sort();
  } catch (err) {
    failWith(
      `Cannot read ${TAURI_KEY_DIR}. Pass --key <path> to point at the private key directly.`,
    );
  }
  if (candidates.length === 0) {
    failWith(
      `No .key file found in ${TAURI_KEY_DIR}. Pass --key <path> to point at the private key directly.`,
    );
  }
  if (candidates.length > 1) {
    const list = candidates.map((f) => `  - ${join(TAURI_KEY_DIR, f)}`).join("\n");
    failWith(
      `Multiple .key files in ${TAURI_KEY_DIR}:\n${list}\nUse --key to pick one explicitly.`,
    );
  }
  const keyPath = join(TAURI_KEY_DIR, candidates[0]);
  const pubPath = keyPath + ".pub";
  if (!existsSync(pubPath)) {
    failWith(`Found ${keyPath} but no matching ${pubPath}. Regenerate the keypair or pass --key.`);
  }
  return verifyPubkeyMatches(keyPath, pubPath, "discovered key");
}

function verifyPubkeyMatches(keyPath, pubPath, label) {
  const pubOnDisk = readFileSync(pubPath, "utf8").trim();
  const pubInConf = readTauriConf().plugins.updater.pubkey;
  if (pubOnDisk !== pubInConf) {
    failWith(
      `Public key mismatch:\n` +
        `  ${pubPath}:\n${pubOnDisk}\n` +
        `  src-tauri/tauri.conf.json#plugins.updater.pubkey:\n${pubInConf}\n` +
        `Either update tauri.conf.json or pass --key to use a different private key.`,
    );
  }
  console.log(`✓ ${label}: ${keyPath} (matches tauri.conf.json pubkey)`);
  return keyPath;
}

function failWith(message) {
  console.error(message);
  process.exit(1);
}

function detectRepo() {
  try {
    const out = execFileSync("git", ["remote", "get-url", "github"], {
      cwd: REPO_ROOT,
      encoding: "utf8",
    }).trim();
    const m = out.match(/github\.com[:/](.+?)\.git$/);
    if (!m) throw new Error(`unexpected remote URL: ${out}`);
    return m[1];
  } catch (err) {
    throw new Error(
      `Cannot read \`github\` remote. Pass --repo <owner/name> or configure it via \`git remote add github ...\`.`,
    );
  }
}

function installerPath(version) {
  return join(BUNDLE_DIR, `CodeBerth_${version}_x64-setup.exe`);
}

function sigPath(version) {
  return installerPath(version) + ".sig";
}

// --- commands ---

function cmdCheck(opts) {
  const v1 = readVersion(TAURI_CONF);
  const v2 = readVersion(PACKAGE_JSON);
  const v3 = readVersion(CARGO_TOML);
  console.log(`tauri.conf.json: ${v1}`);
  console.log(`package.json   : ${v2}`);
  console.log(`Cargo.toml     : ${v3}`);
  if (v1 !== v2 || v1 !== v3) {
    console.error("Version mismatch — fix one of them so all three agree.");
    process.exit(1);
  }
  if (opts.version && opts.version !== v1) {
    console.error(`--version ${opts.version} differs from tauri.conf.json ${v1}`);
    process.exit(1);
  }
  console.log("✓ versions consistent");
}

function cmdBuild(opts) {
  if (opts.skipBuild) {
    const exe = installerPath(opts.version);
    if (!existsSync(exe)) {
      console.error(`--skip-build set but ${exe} not found. Run \`pnpm build:desktop\` first.`);
      process.exit(1);
    }
    console.log("⏭  --skip-build: skipping pnpm build:desktop");
    return;
  }
  // `pnpm build:desktop` runs `tauri build`, which at the end tries to generate
  // updater artifacts and reads TAURI_SIGNING_PRIVATE_KEY from the environment.
  // Without it the build silently fails with "A public key has been found, but
  // no private key." — so we resolve the key and pass it through to the build.
  const buildEnv = signingEnvFrom(opts);
  console.log("→ pnpm install --frozen-lockfile");
  runPnpm(["install", "--frozen-lockfile"], buildEnv);
  console.log("→ pnpm build:desktop");
  runPnpm(["build:desktop"], buildEnv);
  const exe = installerPath(opts.version);
  if (!existsSync(exe)) {
    console.error(`Build finished but ${exe} not found.`);
    process.exit(1);
  }
  console.log(`✓ installer: ${exe}`);
}

// Returns an env object that has both TAURI_SIGNING_PRIVATE_KEY (the key file
// contents) and TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" set, so the build step's
// updater-artifacts generation can sign latest.json inline. See the header
// comment about the Tauri 2.11.4 signer bug.
function signingEnvFrom(opts) {
  const keyPath = resolveKey(opts);
  return {
    ...process.env,
    TAURI_SIGNING_PRIVATE_KEY: readFileSync(keyPath, "utf8").trim(),
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "",
  };
}

function runPnpm(args, env = process.env) {
  // Use corepack to dispatch to the exact pnpm version pinned in
  // package.json#packageManager. Works on any machine with Node 16.10+
  // without requiring pnpm to be on PATH (Node's spawn PATH is sparse).
  try {
    execFileSync("corepack", ["pnpm", ...args], { cwd: REPO_ROOT, stdio: "inherit", env });
  } catch (err) {
    if (err && err.code === "ENOENT") {
      // Fallback: try pnpm directly (developer has it on PATH).
      try {
        execFileSync("pnpm", args, { cwd: REPO_ROOT, stdio: "inherit", env });
        return;
      } catch (err2) {
        if (err2 && err2.code === "ENOENT") {
          console.error(
            `Neither \`corepack\` nor \`pnpm\` is available on PATH. Install Node 16.10+ (corepack) or pnpm 11+ (https://pnpm.io).`,
          );
          process.exit(1);
        }
        throw err2;
      }
    }
    throw err;
  }
}

function cmdSign(opts) {
  const keyPath = resolveKey(opts);
  const exe = installerPath(opts.version);
  if (!existsSync(exe)) {
    console.error(`Installer not found: ${exe}`);
    process.exit(1);
  }
  // Workaround for Tauri 2.11.4 signer bug — see header comment.
  const env = { ...process.env, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "" };
  console.log(`→ tauri signer sign -f ${keyPath} ${exe}`);
  const r = spawnSync(
    "node",
    [
      join(REPO_ROOT, "node_modules", "@tauri-apps", "cli", "tauri.js"),
      "signer",
      "sign",
      "-f",
      keyPath,
      exe,
    ],
    { cwd: REPO_ROOT, stdio: "inherit", env },
  );
  if (r.status !== 0) {
    console.error(`signer exited with code ${r.status}`);
    process.exit(r.status || 1);
  }
  const sig = sigPath(opts.version);
  if (!existsSync(sig)) {
    console.error(`signer reported success but ${sig} was not produced.`);
    process.exit(1);
  }
  console.log(`✓ signature: ${sig}`);
}

// Resolve the private key path:
//   - if --key <path> was passed, use that and verify its .pub matches tauri.conf.json
//   - otherwise auto-discover the single .key under ~/.tauri/
function resolveKey(opts) {
  if (opts.key) {
    const explicit = resolve(opts.key);
    if (!existsSync(explicit)) failWith(`--key ${explicit} not found.`);
    const pub = explicit + ".pub";
    if (!existsSync(pub)) {
      failWith(`--key ${explicit} provided but no matching ${pub} exists for cross-check.`);
    }
    return verifyPubkeyMatches(explicit, pub, "--key");
  }
  return discoverKey();
}

function cmdLatest(opts) {
  const version = opts.version;
  const exe = installerPath(version);
  const sig = sigPath(version);
  if (!existsSync(exe)) throw new Error(`missing installer: ${exe}`);
  if (!existsSync(sig)) throw new Error(`missing signature: ${sig}`);
  const signature = readFileSync(sig, "utf8").trim();
  const repo = opts.repo || detectRepo();
  // Pull notes from the most recent commit subject, fallback to a plain string.
  let notes;
  try {
    notes = execFileSync("git", ["log", "-1", "--pretty=%s"], {
      cwd: REPO_ROOT,
      encoding: "utf8",
    }).trim();
  } catch {
    notes = `CodeBerth v${version}`;
  }
  const latest = {
    version,
    notes,
    pub_date: new Date().toISOString().replace(/\.\d{3}Z$/, "Z"),
    platforms: {
      "windows-x86_64": {
        signature,
        url: `https://github.com/${repo}/releases/latest/download/CodeBerth_${version}_x64-setup.exe`,
      },
    },
  };
  const out = join(BUNDLE_DIR, "latest.json");
  writeFileSync(out, JSON.stringify(latest, null, 2) + "\n");
  // Self-validate.
  const round = JSON.parse(readFileSync(out, "utf8"));
  if (round.platforms["windows-x86_64"].signature !== signature) {
    throw new Error("latest.json round-trip mismatch");
  }
  console.log(`✓ latest.json: ${out}`);
}

function cmdAll(opts) {
  cmdCheck(opts);
  cmdBuild(opts);
  cmdSign(opts);
  cmdLatest(opts);
  console.log("\nDone. Next: upload via gh release create / gh release upload (see release-tagger SKILL).");
}

// --- main ---

const opts = parseArgs(process.argv);
if (opts.version == null) {
  opts.version = readVersion(TAURI_CONF);
}

switch (opts.command) {
  case "check":
    cmdCheck(opts);
    break;
  case "build":
    cmdCheck(opts);
    cmdBuild(opts);
    break;
  case "sign":
    cmdCheck(opts);
    cmdBuild(opts);
    cmdSign(opts);
    break;
  case "latest":
    cmdLatest(opts);
    break;
  case "all":
    cmdAll(opts);
    break;
  default:
    console.error(`Unknown command: ${opts.command}`);
    console.log(USAGE);
    process.exit(2);
}