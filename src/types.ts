// Rust 側の AnalysisResult に対応する型。
export interface LanguageStat {
  name: string;
  count: number;
  percentage: number;
}

export interface FrameworkHit {
  name: string;
  detected_by: string;
}

export type DetectionMethod = "extension" | "filename" | "shebang" | "content";

export interface FileInfo {
  path: string;
  language: string;
  detection: DetectionMethod;
}

// 判定方法の日本語ラベル（拡張子以外は UI に印を付ける）。
export const DETECTION_LABELS: Record<DetectionMethod, string> = {
  extension: "拡張子",
  filename: "ファイル名",
  shebang: "シェバン",
  content: "中身",
};

export interface AnalysisResult {
  root: string;
  total_files: number;
  counted_files: number;
  languages: LanguageStat[];
  frameworks: FrameworkHit[];
  files: FileInfo[];
}

// 言語ごとの代表色（グラフ・バッジ用）。未定義はグレー。
export const LANGUAGE_COLORS: Record<string, string> = {
  JavaScript: "#f1e05a",
  TypeScript: "#3178c6",
  Python: "#3572A5",
  Dart: "#00B4AB",
  Swift: "#F05138",
  HTML: "#e34c26",
  CSS: "#563d7c",
  SCSS: "#c6538c",
  Less: "#1d365d",
  Java: "#b07219",
  Kotlin: "#A97BFF",
  PHP: "#4F5D95",
  Ruby: "#701516",
  Rust: "#dea584",
  Go: "#00ADD8",
  C: "#555555",
  "C++": "#f34b7d",
  "C#": "#178600",
  Vue: "#41b883",
  Svelte: "#ff3e00",
  Shell: "#89e051",
  SQL: "#e38c00",
  R: "#198CE7",
  Lua: "#000080",
  Scala: "#c22d40",
  Elixir: "#6e4a7e",
  Perl: "#0298c3",
  Makefile: "#427819",
  CMake: "#da3434",
};

export function colorFor(lang: string): string {
  return LANGUAGE_COLORS[lang] ?? "#8b949e";
}
