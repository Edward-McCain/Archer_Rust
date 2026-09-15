import { useEffect, useMemo, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  detectFormat,
  listArchive,
  listFormats,
  packArchive,
  unpackArchive,
} from "./api";
import type {
  ArchiveEntry,
  CompressionLevel,
  FormatId,
  FormatInfo,
  Mode,
} from "./types";

function formatBytes(n: number) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 ** 2) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 ** 3) return `${(n / 1024 ** 2).toFixed(1)} MB`;
  return `${(n / 1024 ** 3).toFixed(2)} GB`;
}

function basename(path: string) {
  return path.split(/[/\\]/).pop() ?? path;
}

export default function App() {
  const [mode, setMode] = useState<Mode>("unpack");
  const [formats, setFormats] = useState<FormatInfo[]>([]);
  const [dragging, setDragging] = useState(false);

  const [archivePath, setArchivePath] = useState<string | null>(null);
  const [archiveFormat, setArchiveFormat] = useState<FormatInfo | null>(null);
  const [entries, setEntries] = useState<ArchiveEntry[]>([]);
  const [sources, setSources] = useState<string[]>([]);

  const [packFormat, setPackFormat] = useState<FormatId>("zip");
  const [level, setLevel] = useState<CompressionLevel>("balanced");
  const [password, setPassword] = useState("");
  const [destHint, setDestHint] = useState("");

  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState(0);
  const [progressLabel, setProgressLabel] = useState("");
  const [status, setStatus] = useState("Drop an archive or files to begin");
  const [error, setError] = useState<string | null>(null);

  const packableFormats = useMemo(
    () => formats.filter((f) => f.supportsPacking),
    [formats],
  );

  useEffect(() => {
    listFormats()
      .then(setFormats)
      .catch((err) => setError(String(err)));
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          setDragging(true);
        } else if (event.payload.type === "leave") {
          setDragging(false);
        } else if (event.payload.type === "drop") {
          setDragging(false);
          void handlePaths(event.payload.paths);
        }
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  }, [mode]);

  async function handlePaths(paths: string[]) {
    if (!paths.length) return;
    setError(null);

    if (mode === "unpack") {
      const archive = paths[0];
      setArchivePath(archive);
      setSources([]);
      setStatus(`Opening ${basename(archive)}…`);
      try {
        const info = await detectFormat(archive);
        setArchiveFormat(info);
        const list = await listArchive(archive, password || null);
        setEntries(list);
        setStatus(`${info.displayName} · ${list.length} items`);
      } catch (err) {
        setError(String(err));
        setEntries([]);
        setArchiveFormat(null);
      }
      return;
    }

    setSources(paths);
    setArchivePath(null);
    setEntries([]);
    setArchiveFormat(null);
    setStatus(`${paths.length} item(s) ready to archive`);
  }

  async function pickOpen() {
    const selected = await open({
      multiple: mode === "pack",
      directory: false,
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await handlePaths(paths);
  }

  async function runUnpack() {
    if (!archivePath) return;
    const dest =
      destHint ||
      ((await open({ directory: true, multiple: false })) as string | null);
    if (!dest) return;

    setBusy(true);
    setError(null);
    setProgress(0);
    setProgressLabel("Starting…");
    setStatus("Unpacking…");

    try {
      await unpackArchive({
        archive: archivePath,
        dest,
        password: password || null,
        onProgress: (ev) => {
          if (ev.type === "started") {
            setProgressLabel(
              ev.totalEntries != null
                ? `0 / ${ev.totalEntries}`
                : "Extracting…",
            );
          } else if (ev.type === "entry") {
            const total = entries.length || undefined;
            const pct = total ? Math.min(100, ((ev.index + 1) / total) * 100) : 50;
            setProgress(pct);
            setProgressLabel(ev.path);
          } else {
            setProgress(100);
            setProgressLabel("Done");
          }
        },
      });
      setStatus(`Extracted to ${dest}`);
    } catch (err) {
      setError(String(err));
      setStatus("Unpack failed");
    } finally {
      setBusy(false);
    }
  }

  async function runPack() {
    if (!sources.length) return;
    const ext =
      packableFormats.find((f) => f.id === packFormat)?.extension ?? "zip";
    const suggested = `${basename(sources[0])}.${ext}`;
    const dest =
      destHint ||
      (await save({
        defaultPath: suggested,
        filters: [{ name: "Archive", extensions: [ext] }],
      }));
    if (!dest) return;

    setBusy(true);
    setError(null);
    setProgress(0);
    setProgressLabel("Starting…");
    setStatus("Creating archive…");

    try {
      await packArchive({
        sources,
        dest,
        format: packFormat,
        level,
        password: password || null,
        onProgress: (ev) => {
          if (ev.type === "started") {
            setProgressLabel(
              ev.totalEntries != null
                ? `0 / ${ev.totalEntries}`
                : "Compressing…",
            );
          } else if (ev.type === "entry") {
            const total = sources.length || undefined;
            const pct = total ? Math.min(100, ((ev.index + 1) / total) * 100) : 50;
            setProgress(pct);
            setProgressLabel(ev.path);
          } else {
            setProgress(100);
            setProgressLabel("Done");
          }
        },
      });
      setStatus(`Created ${dest}`);
    } catch (err) {
      setError(String(err));
      setStatus("Pack failed");
    } finally {
      setBusy(false);
    }
  }

  const listItems =
    mode === "unpack"
      ? entries.map((e) => ({
          path: e.path,
          size: e.isDir ? "folder" : formatBytes(e.sizeBytes),
        }))
      : sources.map((p) => ({ path: p, size: "" }));

  return (
    <div className="app">
      <header className="titlebar" data-tauri-drag-region>
        <h1>Archer</h1>
        <div className="modes">
          <button
            className={`mode-btn ${mode === "unpack" ? "active" : ""}`}
            onClick={() => {
              setMode("unpack");
              setStatus("Drop an archive to inspect and extract");
            }}
          >
            Unpack
          </button>
          <button
            className={`mode-btn ${mode === "pack" ? "active" : ""}`}
            onClick={() => {
              setMode("pack");
              setStatus("Drop files or folders to compress");
            }}
          >
            Create
          </button>
        </div>
      </header>

      <main className="content">
        <section className="panel">
          <div className={`dropzone ${dragging ? "active" : ""}`}>
            <div>
              <h2>
                {mode === "unpack"
                  ? "Drop an archive here"
                  : "Drop files & folders"}
              </h2>
              <p>
                {mode === "unpack"
                  ? "ZIP, TAR.*, 7Z, RAR, GZ/BZ2/XZ — or choose a file."
                  : "We'll pack them into the format you pick on the right."}
              </p>
            </div>
          </div>

          <div className="actions">
            <button className="ghost-btn" onClick={pickOpen} disabled={busy}>
              Browse…
            </button>
            {mode === "unpack" ? (
              <button
                className="action-btn primary"
                onClick={runUnpack}
                disabled={busy || !archivePath}
              >
                Extract…
              </button>
            ) : (
              <button
                className="action-btn primary"
                onClick={runPack}
                disabled={busy || sources.length === 0}
              >
                Create archive…
              </button>
            )}
          </div>

          <div className="meta">
            {archiveFormat && (
              <span className="chip">{archiveFormat.displayName}</span>
            )}
            {archivePath && (
              <span className="chip">{basename(archivePath)}</span>
            )}
            {mode === "pack" && sources.length > 0 && (
              <span className="chip">{sources.length} sources</span>
            )}
          </div>

          <ul className="file-list">
            {listItems.length === 0 ? (
              <li>
                <span className="path">No items yet</span>
                <span className="size">—</span>
              </li>
            ) : (
              listItems.map((item) => (
                <li key={item.path}>
                  <span className="path" title={item.path}>
                    {item.path}
                  </span>
                  <span className="size">{item.size}</span>
                </li>
              ))
            )}
          </ul>

          <div className="progress-wrap">
            <div className="progress-track">
              <div className="progress-bar" style={{ width: `${progress}%` }} />
            </div>
            <div className="progress-label">{progressLabel}</div>
          </div>

          <p className={`status ${error ? "error" : ""}`}>
            {error ?? status}
          </p>
        </section>

        <aside className="panel">
          <h2 style={{ margin: 0, fontSize: 18 }}>Options</h2>
          <p className="sidebar-note">
            Light and dark follow your macOS appearance. Heavy work runs off the
            UI thread; progress streams live from Rust.
          </p>

          <div className="field-grid">
            {mode === "pack" && (
              <>
                <div className="field">
                  <label htmlFor="format">Format</label>
                  <select
                    id="format"
                    value={packFormat}
                    onChange={(e) => setPackFormat(e.target.value as FormatId)}
                  >
                    {packableFormats.map((f) => (
                      <option key={f.id} value={f.id}>
                        {f.displayName}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="field">
                  <label htmlFor="level">Compression</label>
                  <select
                    id="level"
                    value={level}
                    onChange={(e) =>
                      setLevel(e.target.value as CompressionLevel)
                    }
                  >
                    <option value="fast">Fast</option>
                    <option value="balanced">Balanced</option>
                    <option value="maximum">Maximum</option>
                  </select>
                </div>
              </>
            )}

            <div className="field full">
              <label htmlFor="password">Password (ZIP / 7Z / RAR)</label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Optional"
                autoComplete="off"
              />
            </div>

            <div className="field full">
              <label htmlFor="dest">Destination override (optional)</label>
              <input
                id="dest"
                value={destHint}
                onChange={(e) => setDestHint(e.target.value)}
                placeholder={
                  mode === "unpack"
                    ? "/Users/you/Desktop/extracted"
                    : "/Users/you/Desktop/archive.zip"
                }
              />
            </div>
          </div>
        </aside>
      </main>
    </div>
  );
}
