import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  ArchiveEntry,
  ArchiveProgress,
  CompressionLevel,
  FormatId,
  FormatInfo,
} from "./types";

export function listFormats() {
  return invoke<FormatInfo[]>("list_formats");
}

export function detectFormat(path: string) {
  return invoke<FormatInfo>("detect_format", { path });
}

export function listArchive(path: string, password?: string | null) {
  return invoke<ArchiveEntry[]>("list_archive", {
    path,
    password: password || null,
  });
}

export async function packArchive(args: {
  sources: string[];
  dest: string;
  format: FormatId;
  level?: CompressionLevel | null;
  password?: string | null;
  onProgress: (ev: ArchiveProgress) => void;
}) {
  const onEvent = new Channel<ArchiveProgress>();
  onEvent.onmessage = args.onProgress;
  await invoke("pack_archive", {
    sources: args.sources,
    dest: args.dest,
    format: args.format,
    level: args.level ?? null,
    password: args.password || null,
    onEvent,
  });
}

export async function unpackArchive(args: {
  archive: string;
  dest: string;
  password?: string | null;
  overwrite?: boolean;
  onProgress: (ev: ArchiveProgress) => void;
}) {
  const onEvent = new Channel<ArchiveProgress>();
  onEvent.onmessage = args.onProgress;
  await invoke("unpack_archive", {
    archive: args.archive,
    dest: args.dest,
    password: args.password || null,
    overwrite: args.overwrite ?? true,
    onEvent,
  });
}
