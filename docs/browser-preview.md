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
