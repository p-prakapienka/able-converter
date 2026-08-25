# Browser preview architecture

The browser interface is a local adapter over the same conversion pipeline used by
native tools:

```text
selected .als bytes
  -> LiveParser
  -> LiveToInternalMapper
  -> InternalToNoteMapper
  -> ProjectPreviewFactory
  -> source/target canvas views and compatibility report
```

`BrowserAdapter` is the only WebAssembly-specific Rust object. It accepts bytes from
the browser File API, coordinates the stateless mapper services, and returns a Serde
preview value to JavaScript. It does not receive filesystem paths or perform network
requests.

`ProjectPreviewFactory` is a stateless, platform-neutral factory covered by native
Rust tests. It constructs a composite preview from the canonical source project and
the exact typed Note output, aligning their tracks and clips and calculating the
summary and compatibility report. The source piano roll marks:

- ordinary canonical notes as `source`;
- notes with lossy properties such as probability as `lossy`;
- notes omitted from the verified Note subset as `omitted`.

The target piano roll reads the mapped `NoteProject`, so it represents the notes that
the exporter will serialize rather than independently reimplementing conversion.

`frontend/app.js` owns browser file selection, clip navigation, and HTML canvas
rendering. The same frontend assets can later be hosted by the Tauri desktop webview;
no DOM or canvas types leak into the parser, canonical model, mappers, or exporter.

## Testing the CI artifact on Windows

Each pull request publishes three downloadable artifacts:

- `browser-preview-standalone` is a single self-contained HTML file. Download it,
  double-click it, and drop an `.als` file onto the page. It needs no local HTTP
  server: the WebAssembly module is embedded and initialised from memory.
- `browser-preview` contains the served frontend folder with its compiled
  WebAssembly package, for development and debugging.
- `windows-tools` contains release builds of the developer `inspect` and `convert`
  utilities as `.exe` files.

To build the standalone file locally:

```powershell
wasm-pack build --target web --out-dir target/browser-preview/pkg
node src/web/frontend/bundle-single-file.mjs
```

This writes `target/able-converter-preview.html`. Everything under `target/` is a
local build artifact and should not be committed. Open the page directly in the
browser, then drop an `.als` file onto it. The browser reads the selected bytes
locally and does not upload the Set.

To test the served folder instead, stage the static assets next to the compiled
package and serve the staged folder from a local HTTP server. It cannot be opened
directly from disk because browsers do not load external modules or WebAssembly
over `file://` URLs:

```powershell
wasm-pack build --target web --out-dir target/browser-preview/pkg
node src/web/frontend/stage-preview.mjs --out target/browser-preview
cd target/browser-preview
python -m http.server 8000
```

Open `http://localhost:8000`, then drop an `.als` file onto the page.

To test the native behaviour, download and extract `windows-tools`, then run:

```powershell
.\inspect.exe path\to\project.als
.\convert.exe path\to\project.als path\to\output.ablbundle
```
