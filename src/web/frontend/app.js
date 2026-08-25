import init, { BrowserAdapter } from "./pkg/able_converter.js";

const elements = {
  dropZone: document.querySelector("#dropZone"),
  fileInput: document.querySelector("#fileInput"),
  status: document.querySelector("#status"),
  workspace: document.querySelector("#workspace"),
  summary: document.querySelector("#summary"),
  clipList: document.querySelector("#clipList"),
  selectedClipLabel: document.querySelector("#selectedClipLabel"),
  sourceRoll: document.querySelector("#sourceRoll"),
  targetRoll: document.querySelector("#targetRoll"),
  diagnosticCounts: document.querySelector("#diagnosticCounts"),
  diagnostics: document.querySelector("#diagnostics"),
};

const state = {
  adapter: null,
  preview: null,
  selectedSourceId: null,
};

await init();
state.adapter = new BrowserAdapter();

elements.dropZone.addEventListener("click", () => elements.fileInput.click());
elements.dropZone.addEventListener("keydown", (event) => {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    elements.fileInput.click();
  }
});
elements.fileInput.addEventListener("change", () => loadFile(elements.fileInput.files[0]));

for (const eventName of ["dragenter", "dragover"]) {
  elements.dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    elements.dropZone.classList.add("dragging");
  });
}

for (const eventName of ["dragleave", "drop"]) {
  elements.dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    elements.dropZone.classList.remove("dragging");
  });
}

elements.dropZone.addEventListener("drop", (event) => loadFile(event.dataTransfer.files[0]));
window.addEventListener("resize", () => renderSelectedClip());

async function loadFile(file) {
  if (!file) return;
  if (!file.name.toLowerCase().endsWith(".als")) {
    setStatus("Choose an Ableton Live .als file.", true);
    return;
  }

  setStatus(`Reading ${file.name}…`);
  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    state.preview = state.adapter.loadProject(bytes);
    state.selectedSourceId = firstSourceClip()?.id ?? null;
    renderProject();
    setStatus(`${file.name} inspected locally.`);
  } catch (error) {
    elements.workspace.hidden = true;
    setStatus(String(error), true);
  }
}

function renderProject() {
  elements.workspace.hidden = false;
  renderSummary();
  renderClipList();
  renderCompatibility();
  renderSelectedClip();
}

function renderSummary() {
  const summary = state.preview.summary;
  const values = [
    ["Tempo", summary.tempo ? `${formatNumber(summary.tempo)} BPM` : "Unknown"],
    ["Source tracks", summary.sourceTrackCount],
    ["Note tracks", summary.targetTrackCount],
    ["Scenes", summary.sceneCount],
    ["Source clips", summary.sourceClipCount],
    ["Mapped clips", summary.targetClipCount],
  ];

  elements.summary.replaceChildren(
    ...values.map(([label, value]) => {
      const card = document.createElement("div");
      card.className = "summary-card";
      card.innerHTML = `<span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong>`;
      return card;
    }),
  );
}

function renderClipList() {
  const rows = state.preview.sourceTracks.map((track) => {
    const row = document.createElement("div");
    row.className = "track-row";

    const trackName = document.createElement("div");
    trackName.className = "track-name";
    trackName.textContent = track.name || track.id;

    const buttons = document.createElement("div");
    buttons.className = "clip-buttons";
    if (track.clips.length === 0) {
      const empty = document.createElement("span");
      empty.className = "track-name";
      empty.textContent = "No Session clips";
      buttons.append(empty);
    }

    for (const clip of track.clips) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "clip-button";
      button.classList.toggle("selected", clip.id === state.selectedSourceId);
      const scene = state.preview.scenes.find((item) => item.index === clip.sceneIndex);
      button.innerHTML = `${escapeHtml(clip.name || "Untitled clip")}<small>${escapeHtml(scene?.name ?? `Scene ${clip.sceneIndex + 1}`)}</small>`;
      button.addEventListener("click", () => {
        state.selectedSourceId = clip.id;
        renderClipList();
        renderSelectedClip();
      });
      buttons.append(button);
    }

    row.append(trackName, buttons);
    return row;
  });

  elements.clipList.replaceChildren(...rows);
}

function renderSelectedClip() {
  if (!state.preview) return;
  const sourceClip = findSourceClip(state.selectedSourceId);
  const targetClip = findTargetClip(state.selectedSourceId);
  elements.selectedClipLabel.textContent = sourceClip
    ? `${sourceClip.trackId} · scene ${sourceClip.sceneIndex + 1}`
    : "No clip selected";

  const notes = [...(sourceClip?.notes ?? []), ...(targetClip?.notes ?? [])];
  const pitchRange = notes.length
    ? {
        minimum: Math.max(0, Math.min(...notes.map((note) => note.pitch)) - 2),
        maximum: Math.min(127, Math.max(...notes.map((note) => note.pitch)) + 2),
      }
    : { minimum: 48, maximum: 72 };

  drawPianoRoll(elements.sourceRoll, sourceClip, pitchRange, "source");
  drawPianoRoll(elements.targetRoll, targetClip, pitchRange, "target");
}

function drawPianoRoll(canvas, clip, pitchRange, side) {
  const ratio = window.devicePixelRatio || 1;
  const width = Math.max(320, Math.floor(canvas.clientWidth));
  const height = Math.max(260, Math.floor(canvas.clientHeight));
  canvas.width = Math.floor(width * ratio);
  canvas.height = Math.floor(height * ratio);
  const context = canvas.getContext("2d");
  context.scale(ratio, ratio);
  context.clearRect(0, 0, width, height);
  context.fillStyle = "#0b1018";
  context.fillRect(0, 0, width, height);

  if (!clip) {
    context.fillStyle = "#78869a";
    context.font = "13px ui-monospace, monospace";
    context.fillText(side === "target" ? "Not mapped" : "Select a clip", 18, 30);
    return;
  }

  const margin = { left: 42, right: 12, top: 14, bottom: 24 };
  const plotWidth = width - margin.left - margin.right;
  const plotHeight = height - margin.top - margin.bottom;
  const start = clip.start;
  const end = Math.max(start + 1, clip.end);
  const pitchCount = Math.max(1, pitchRange.maximum - pitchRange.minimum + 1);
  const xForBeat = (beat) => margin.left + ((beat - start) / (end - start)) * plotWidth;
  const yForPitch = (pitch) =>
    margin.top + ((pitchRange.maximum - pitch) / pitchCount) * plotHeight;
  const rowHeight = plotHeight / pitchCount;

  if (clip.loopEnabled) {
    const loopStart = Math.max(start, clip.loopStart);
    const loopEnd = Math.min(end, clip.loopEnd);
    context.fillStyle = "rgb(76 224 210 / 5%)";
    context.fillRect(
      xForBeat(loopStart),
      margin.top,
      Math.max(0, xForBeat(loopEnd) - xForBeat(loopStart)),
      plotHeight,
    );
  }

  for (let pitch = pitchRange.minimum; pitch <= pitchRange.maximum; pitch += 1) {
    const y = yForPitch(pitch);
    context.strokeStyle = pitch % 12 === 0 ? "#334157" : "#1a2331";
    context.lineWidth = pitch % 12 === 0 ? 1 : 0.5;
    context.beginPath();
    context.moveTo(margin.left, y);
    context.lineTo(width - margin.right, y);
    context.stroke();
    if (pitch % 12 === 0) {
      context.fillStyle = "#68768a";
      context.font = "10px ui-monospace, monospace";
      context.fillText(`C${Math.floor(pitch / 12) - 1}`, 9, y + 3);
    }
  }

  for (let beat = Math.ceil(start); beat <= end; beat += 1) {
    const x = xForBeat(beat);
    context.strokeStyle = beat % 4 === 0 ? "#3a475d" : "#222c3b";
    context.lineWidth = beat % 4 === 0 ? 1 : 0.5;
    context.beginPath();
    context.moveTo(x, margin.top);
    context.lineTo(x, height - margin.bottom);
    context.stroke();
    context.fillStyle = "#68768a";
    context.font = "10px ui-monospace, monospace";
    context.fillText(formatNumber(beat), x + 3, height - 8);
  }

  for (const note of clip.notes) {
    const x = xForBeat(note.start);
    const noteWidth = Math.max(3, xForBeat(note.start + note.duration) - x);
    const y = yForPitch(note.pitch);
    const noteHeight = Math.max(3, rowHeight - 1.5);
    const colors = {
      source: "#9d82ff",
      lossy: "#ffbf69",
      omitted: "#ff6b7a",
      mapped: "#4ce0d2",
    };
    context.globalAlpha = note.state === "omitted" ? 0.55 : 0.92;
    context.fillStyle = note.state === "omitted" ? "transparent" : colors[note.state];
    context.strokeStyle = colors[note.state];
    context.lineWidth = 1;
    context.setLineDash(note.state === "omitted" ? [4, 3] : []);
    if (note.state !== "omitted") context.fillRect(x, y, noteWidth, noteHeight);
    context.strokeRect(x, y, noteWidth, noteHeight);
  }
  context.setLineDash([]);
  context.globalAlpha = 1;
}

function renderCompatibility() {
  const report = state.preview.compatibility;
  elements.diagnosticCounts.innerHTML = `
    <span class="count warning">${report.warningCount} warnings</span>
    <span class="count error">${report.errorCount} errors</span>
  `;

  if (report.diagnostics.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-report";
    empty.textContent = "No compatibility issues were reported for the verified subset.";
    elements.diagnostics.replaceChildren(empty);
    return;
  }

  elements.diagnostics.replaceChildren(
    ...report.diagnostics.map((diagnostic) => {
      const row = document.createElement("div");
      row.className = `diagnostic ${diagnostic.severity}`;
      row.innerHTML = `
        <span class="severity">${escapeHtml(diagnostic.severity.toUpperCase())}</span>
        <code>${escapeHtml(diagnostic.code)} · ${escapeHtml(diagnostic.sourceId)}</code>
        <span>${escapeHtml(diagnostic.message)}</span>
      `;
      return row;
    }),
  );
}

function firstSourceClip() {
  return state.preview?.sourceTracks.flatMap((track) => track.clips)[0] ?? null;
}

function findSourceClip(sourceId) {
  return (
    state.preview?.sourceTracks
      .flatMap((track) => track.clips)
      .find((clip) => clip.id === sourceId) ?? null
  );
}

function findTargetClip(sourceId) {
  return (
    state.preview?.targetTracks
      .flatMap((track) => track.clips)
      .find((clip) => clip.sourceId === sourceId) ?? null
  );
}

function setStatus(message, error = false) {
  elements.status.textContent = message;
  elements.status.classList.toggle("error", error);
}

function formatNumber(value) {
  return new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 }).format(value);
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}
