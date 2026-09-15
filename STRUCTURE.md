# Project layout (Stage 1)

```text
Archer_Rust/
├── .gitignore
├── Cargo.toml                 # workspace root
├── README.md
├── task.md                    # product requirements
├── archiver-core/             # UI-free archive library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # get_archiver / archiver_for_path
│       ├── error.rs           # ArchiverError
│       ├── format.rs          # ArchiveFormat + detect
│       ├── types.rs           # Archiver trait, options, progress
│       └── handlers/
│           ├── mod.rs
│           ├── zip_handler.rs # pack / unpack / list
│           └── tar_handler.rs # tar + gz/bz2/xz
│
├── src-tauri/                 # (Stage 2) Tauri backend + commands
├── src/                       # (Stage 2) React + TS UI
├── package.json               # (Stage 2)
├── vite.config.ts             # (Stage 2)
└── index.html                 # (Stage 2)
```

## Dependency plan

### `archiver-core`
- `zip`, `tar`, `flate2`, `bzip2`, `xz2`
- `sevenz-rust`, `unrar`
- `thiserror`, `walkdir`

### `src-tauri` (next stage)
- `tauri` 2.x, `tauri-plugin-dialog`, `tauri-plugin-notification`, `tauri-plugin-fs`
- `archiver_core` (path dependency)
- `serde`, `serde_json`, `tokio`

### Frontend (next stage)
- React 18+, TypeScript, Vite
- `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-notification`
