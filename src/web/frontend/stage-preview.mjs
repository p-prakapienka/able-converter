// Stages the served browser preview outside the sources.
//
// Copies the static frontend assets (markup, logic, styles) next to the
// compiled wasm-pack package so the preview can be served from target/ while
// src/ stays free of build output. The ./pkg/ import in app.js resolves
// because the package lives alongside the staged assets.
//
// Usage:
//   node stage-preview.mjs --out target/browser-preview
// The default resolves to <repository root>/target/browser-preview.

import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..", "..", "..");

const STATIC_ASSETS = ["index.html", "app.js", "styles.css"];

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith("--")) continue;
    const equals = token.indexOf("=");
    if (equals === -1) {
      args[token.slice(2)] = argv[index + 1];
      index += 1;
    } else {
      args[token.slice(2, equals)] = token.slice(equals + 1);
    }
  }
  return args;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const outDir = resolve(args.out ?? resolve(repoRoot, "target", "browser-preview"));

  mkdirSync(outDir, { recursive: true });
  for (const asset of STATIC_ASSETS) {
    copyFileSync(resolve(scriptDir, asset), resolve(outDir, asset));
  }
  console.log(`staging: copied ${STATIC_ASSETS.length} assets to ${outDir}`);
}

main();
