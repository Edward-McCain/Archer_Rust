# Archer

Нативный архиватор/разархиватор для macOS на **Rust + Tauri 2**.

Репозиторий: [Edward-McCain/Archer_Rust](https://github.com/Edward-McCain/Archer_Rust)

## Цель

Приложение для создания и распаковки архивов с современным UI в духе macOS:
drag & drop, прогресс, просмотр содержимого, пароли, светлая/тёмная тема.

## Стек

| Слой | Технология |
|------|------------|
| Core | Rust (`archiver-core`) |
| Backend | Tauri 2.x (`src-tauri`) |
| Frontend | Vite + React + TypeScript (`src`) |
| Сборка | cargo + tauri-cli → `.app` / `.dmg` |

## Структура проекта

```text
.
├── Cargo.toml              # Cargo workspace
├── archiver-core/          # Чистая Rust-библиотека архивации
├── src-tauri/              # Tauri backend + commands
├── src/                    # React + TypeScript UI
├── package.json
├── vite.config.ts
└── task.md
```

## Форматы

| Формат | Pack | Unpack | Пароль |
|--------|------|--------|--------|
| ZIP | ✅ | ✅ | ✅ |
| TAR / TAR.GZ / TAR.BZ2 / TAR.XZ | ✅ | ✅ | — |
| 7Z | ⏳ | ⏳ | ⏳ |
| RAR | — | ⏳ | ⏳ |
| GZ / BZ2 / XZ (single) | ⏳ | ⏳ | — |

✅ — реализовано в `archiver-core` · ⏳ — следующий этап

## Этапы разработки

1. ~~Структура workspace + ядро (ZIP/TAR)~~
2. ~~Scaffold Tauri 2 + связка с `archiver-core`~~ — текущий
3. Доработка core: 7Z, RAR, single compress
4. Tauri-команды, progress events, ошибки
5. UI: DnD, список файлов, прогресс, темы
6. История, уведомления, упаковка `.app` / `.dmg`

## Быстрый старт

```bash
# Rust (rustup) + Node.js
npm install
cargo test -p archiver_core
npm run tauri dev
```

## Лицензия

MIT
