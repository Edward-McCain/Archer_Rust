# Project layout

```text
Archer_Rust/
├── .gitignore
├── Cargo.toml                 # workspace: archiver-core + src-tauri
├── README.md
├── STRUCTURE.md
├── task.md
├── package.json               # Vite + React + @tauri-apps/*
├── vite.config.ts
├── index.html
├── tsconfig.json
├── archiver-core/             # UI-free archive library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       ├── format.rs
│       ├── types.rs
│       └── handlers/
├── src-tauri/                 # Tauri 2 backend
│   ├── Cargo.toml             # depends on archiver_core
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── icons/
│   └── src/
│       ├── main.rs
│       └── lib.rs             # commands: greet, detect_format
└── src/                       # React + TypeScript frontend
    ├── App.tsx
    ├── App.css
    └── main.tsx
```

## Dependency plan

### `archiver-core`
- `zip`, `tar`, `flate2`, `bzip2`, `xz2`
- `sevenz-rust`, `unrar`
- `thiserror`, `walkdir`

### `src-tauri`
- `tauri` 2.x, `tauri-plugin-opener`
- `archiver_core` (path)
- `serde`, `serde_json`
- Next: `tauri-plugin-dialog`, `tauri-plugin-notification`, `tauri-plugin-fs`

### Frontend
- React 19, TypeScript, Vite
- `@tauri-apps/api`, `@tauri-apps/plugin-opener`
- Next: dialog / notification plugins, DnD UI
