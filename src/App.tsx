import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [formatMsg, setFormatMsg] = useState("");
  const [name, setName] = useState("");
  const [path, setPath] = useState("archive.tar.gz");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  async function detect() {
    try {
      const format = await invoke<string>("detect_format", { path });
      setFormatMsg(`Detected: ${format}`);
    } catch (err) {
      setFormatMsg(`Error: ${String(err)}`);
    }
  }

  return (
    <main className="container">
      <h1>Archer</h1>
      <p>macOS archive manager — Tauri + archiver-core scaffold</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

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
          placeholder="path/to/archive.zip"
        />
        <button type="submit">Detect format</button>
      </form>
      <p>{formatMsg}</p>
    </main>
  );
}

export default App;
