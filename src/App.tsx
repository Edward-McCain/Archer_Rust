import { useEffect, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import "./App.css";

type FormatInfo = {
  id: string;
  extension: string;
  displayName: string;
  supportsPacking: boolean;
};

type ArchiveProgress =
  | { type: "started"; totalEntries: number | null }
  | { type: "entry"; path: string; index: number }
  | { type: "finished" };

function App() {
  const [formats, setFormats] = useState<FormatInfo[]>([]);
  const [path, setPath] = useState("archive.zip");
  const [status, setStatus] = useState("Ready");
  const [progress, setProgress] = useState("");

  useEffect(() => {
    invoke<FormatInfo[]>("list_formats")
      .then(setFormats)
      .catch((err) => setStatus(`Error: ${String(err)}`));
  }, []);

  async function detect() {
    try {
      const info = await invoke<FormatInfo>("detect_format", { path });
      setStatus(`Detected ${info.displayName} (.${info.extension})`);
    } catch (err) {
      setStatus(`Error: ${String(err)}`);
    }
  }

  async function list() {
    try {
      const entries = await invoke<
        { path: string; sizeBytes: number; isDir: boolean }[]
      >("list_archive", { path, password: null });
      setStatus(`Entries: ${entries.length}`);
      setProgress(entries.slice(0, 8).map((e) => e.path).join("\n"));
    } catch (err) {
      setStatus(`Error: ${String(err)}`);
    }
  }

  async function unpackDemo() {
    const onEvent = new Channel<ArchiveProgress>();
    onEvent.onmessage = (message) => {
      if (message.type === "started") {
        setProgress(`Started (total: ${message.totalEntries ?? "?"})`);
      } else if (message.type === "entry") {
        setProgress(`#${message.index}: ${message.path}`);
      } else {
        setProgress("Finished");
      }
    };

    try {
      setStatus("Unpacking…");
      await invoke("unpack_archive", {
        archive: path,
        dest: "/tmp/archer-extract",
        password: null,
        overwrite: true,
        onEvent,
      });
      setStatus("Unpack complete → /tmp/archer-extract");
    } catch (err) {
      setStatus(`Error: ${String(err)}`);
    }
  }

  return (
    <main className="container">
      <h1>Archer</h1>
      <p>Stage 4 — Tauri commands + progress channel</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          detect();
        }}
      >
        <input
          value={path}
          onChange={(e) => setPath(e.currentTarget.value)}
          placeholder="/path/to/archive.zip"
        />
        <button type="submit">Detect</button>
        <button type="button" onClick={list}>
          List
        </button>
        <button type="button" onClick={unpackDemo}>
          Unpack
        </button>
      </form>

      <p>{status}</p>
      <pre style={{ textAlign: "left", whiteSpace: "pre-wrap" }}>{progress}</pre>

      <h3>Formats ({formats.length})</h3>
      <ul style={{ textAlign: "left" }}>
        {formats.map((f) => (
          <li key={f.id}>
            {f.displayName} — .{f.extension}
            {f.supportsPacking ? "" : " (unpack only)"}
          </li>
        ))}
      </ul>
    </main>
  );
}

export default App;
