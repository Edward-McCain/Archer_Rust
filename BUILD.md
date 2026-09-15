# Build Archer (.app + .dmg)

```bash
# Dev
npm install
npm run tauri dev

# Release bundle for macOS
npm run tauri build
```

Artifacts land in:

```text
src-tauri/target/release/bundle/macos/Archer.app
src-tauri/target/release/bundle/dmg/Archer_0.1.0_*.dmg 
```

## Notes

- Requires Rust (rustup), Node.js, and Xcode Command Line Tools.
- Unsigned local builds work for personal use; Gatekeeper may require
  right-click → Open the first time.
- Bundle targets are configured in `src-tauri/tauri.conf.json`
  (`app` + `dmg`, macOS 11+).
