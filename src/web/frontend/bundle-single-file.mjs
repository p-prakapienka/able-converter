// Bundles the served browser preview into a single self-contained HTML file
// that runs directly from disk (double-click) without a local HTTP server.
//
// Browsers block external ES module imports and fetch() on file:// URLs, so
// this script inlines the wasm-bindgen glue, the compiled .wasm (base64), the
// stylesheet, the markup, and the frontend logic into one inline module
// script. The module is initialised from embedded bytes and never fetched.
//
// The transform is strict on purpose: there is no Cargo.lock pinning the
// wasm-bindgen output shape, so any unexpected glue structure fails the build
// loudly instead of shipping a broken page.
//
// Usage:
//   node bundle-single-file.mjs --pkg target/browser-preview/pkg --out target/able-converter-preview.html
// Defaults resolve under the repository target/ folder, keeping build output
// out of src/.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..", "..", "..");

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

function readInput(path, label) {
  try {
    return readFileSync(path, "utf8");
  } catch (error) {
    throw new Error(`bundler: cannot read ${label} at ${path}: ${error.message}`);
  }
}

// Removes module import/export syntax from the wasm-bindgen glue so it can be
// inlined. The public names stay declared in the surrounding scope and are
// captured explicitly by the caller.
function stripModuleSyntax(glue) {
  if (/^\s*import\b/m.test(glue) || glue.includes("import(")) {
    throw new Error(
      "bundler: wasm-bindgen glue contains import statements and cannot be inlined",
    );
  }

  let code = glue;
  let defaultInit = null;
  const defaultMatch = code.match(/export\s+default\s+([A-Za-z_$][\w$]*)\s*;/);
  if (defaultMatch) {
    defaultInit = defaultMatch[1];
    code = code.replace(defaultMatch[0], "");
  }
  code = code
    .replace(/export\s+default\s+async\s+function\s+([A-Za-z_$][\w$]*)/, (_, name) => {
      defaultInit = name;
      return `async function ${name}`;
    })
    .replace(/export\s+default\s+function\s+([A-Za-z_$][\w$]*)/, (_, name) => {
      defaultInit = name;
      return `function ${name}`;
    })
    .replace(/export\s+default\s+class\s+([A-Za-z_$][\w$]*)/, "class $1")
    .replace(/export\s*\{[^}]*\}\s*;?/g, "")
    .replace(/export\s+async\s+function\s+/g, "async function ")
    .replace(/export\s+function\s+/g, "function ")
    .replace(/export\s+class\s+/g, "class ")
    .replace(/export\s+(const|let|var)\s+/g, "$1 ");

  if (/^\s*export\b/m.test(code)) {
    throw new Error("bundler: unsupported export statement remains in wasm-bindgen glue");
  }
  const usesInitSync = /\binitSync\b/.test(code);
  if (!usesInitSync && defaultInit === null) {
    throw new Error("bundler: glue exposes neither initSync nor a default init function");
  }
  // Current wasm-bindgen deprecates positional initSync arguments; the options
  // object form initialises from memory without any fetch. Older glue without
  // initSync keeps the positional bytes form.
  return {
    code,
    initExpression: usesInitSync ? "initSync" : defaultInit,
    initArguments: usesInitSync
      ? "{ module: __ableConverterWasmBytes() }"
      : "__ableConverterWasmBytes()",
  };
}

function transformApp(app, initArguments) {
  const importMatches = app.match(/\.\/pkg\/able_converter\.js/g) ?? [];
  if (importMatches.length !== 1) {
    throw new Error("bundler: app.js must import the wasm glue exactly once");
  }
  let code = app.replace(/^\s*import\s+[^;]*from\s*"\.\/pkg\/able_converter\.js";\s*$/m, "");
  if (code === app) {
    throw new Error("bundler: app.js glue import line was not recognised");
  }
  if (!code.includes("await init();")) {
    throw new Error("bundler: app.js init call was not recognised");
  }
  code = code.replace("await init();", `await init(${initArguments});`);
  if (code.includes("./pkg/") || /from\s*["']\.\//.test(code)) {
    throw new Error("bundler: app.js still references local files after the transform");
  }
  return code;
}

function extractMarkup(markup) {
  const bodyStart = markup.indexOf("<body>");
  const scriptTag = markup.indexOf('<script type="module" src="./app.js">');
  if (bodyStart === -1 || scriptTag === -1 || scriptTag < bodyStart) {
    throw new Error("bundler: index.html structure was not recognised");
  }
  const body = markup.slice(bodyStart + "<body>".length, scriptTag);
  if (body.includes("<script") || body.includes("src=") || body.includes("href=")) {
    throw new Error("bundler: index.html body references external files");
  }
  return body;
}

function extractTitle(markup) {
  const match = markup.match(/<title>([^<]*)<\/title>/);
  return match ? match[1] : "Able Converter · Project Inspector";
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const pkgDir = resolve(args.pkg ?? resolve(repoRoot, "target", "browser-preview", "pkg"));
  const outFile = resolve(args.out ?? resolve(repoRoot, "target", "able-converter-preview.html"));

  const glue = readInput(resolve(pkgDir, "able_converter.js"), "wasm-bindgen glue");
  const wasmBytes = (() => {
    try {
      return readFileSync(resolve(pkgDir, "able_converter_bg.wasm"));
    } catch (error) {
      throw new Error(`bundler: cannot read compiled wasm: ${error.message}`);
    }
  })();
  const markup = readInput(resolve(scriptDir, "index.html"), "page markup");
  const styles = readInput(resolve(scriptDir, "styles.css"), "stylesheet");
  const app = readInput(resolve(scriptDir, "app.js"), "frontend logic");

  if (styles.includes("</style")) {
    throw new Error("bundler: stylesheet cannot be inlined safely");
  }

  const stripped = stripModuleSyntax(glue);
  const appCode = transformApp(app, stripped.initArguments);
  const body = extractMarkup(markup);

  const script = `{
${stripped.code}
globalThis.__ableConverterInit = ${stripped.initExpression};
globalThis.__ableConverterBrowserAdapter = BrowserAdapter;
}

const __ableConverterWasmBytes = () =>
  Uint8Array.from(atob("${wasmBytes.toString("base64")}"), (character) =>
    character.charCodeAt(0),
  );
const init = globalThis.__ableConverterInit;
const BrowserAdapter = globalThis.__ableConverterBrowserAdapter;
${appCode}`;

  if (script.toLowerCase().includes("</script")) {
    throw new Error("bundler: inline script cannot be embedded safely");
  }

  const page = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="color-scheme" content="dark" />
    <title>${extractTitle(markup)}</title>
    <style>
${styles}
    </style>
  </head>
  <body>
${body}    <script type="module">
${script}
    </script>
  </body>
</html>
`;

  mkdirSync(dirname(outFile), { recursive: true });
  writeFileSync(outFile, page);
  console.log(`bundler: wrote ${outFile} (${page.length} characters)`);
}

main();
