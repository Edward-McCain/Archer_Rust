export type FormatInfo = {
  id: FormatId;
  extension: string;
  displayName: string;
  supportsPacking: boolean;
};

export type FormatId =
  | "zip"
  | "tar"
  | "tarGz"
  | "tarBz2"
  | "tarXz"
  | "gzip"
  | "bzip2"
  | "xz"
  | "sevenZ"
  | "rar";

export type CompressionLevel = "fast" | "balanced" | "maximum";

export type ArchiveEntry = {
  path: string;
  sizeBytes: number;
  isDir: boolean;
};

export type ArchiveProgress =
  | { type: "started"; totalEntries: number | null }
  | { type: "entry"; path: string; index: number }
  | { type: "finished" };

export type Mode = "unpack" | "pack";
