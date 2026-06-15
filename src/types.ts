// Rust 側の AnalysisResult に対応する型。
export interface LanguageStat {
  name: string;
  count: number;
  percentage: number;
}

export type TechCategory = "framework" | "backend" | "database";

export interface TechHit {
  name: string;
  category: TechCategory;
  detected_by: string;
}

// カテゴリの表示情報（見出し・アイコン・アクセント色）。
export const CATEGORY_META: Record<
  TechCategory,
  { label: string; icon: string; accent: string }
> = {
  framework: { label: "フレームワーク", icon: "🧩", accent: "#7c8cff" },
  backend: { label: "バックエンド・サービス", icon: "☁️", accent: "#3fb950" },
  database: { label: "データベース・ORM", icon: "🗄️", accent: "#d29922" },
};
export const CATEGORY_ORDER: TechCategory[] = ["framework", "backend", "database"];

// 主要サービスのブランド色（バッジ用）。未定義はカテゴリ色を使う。
export const TECH_COLORS: Record<string, string> = {
  // frameworks
  "Next.js": "#000000",
  React: "#61dafb",
  Vue: "#41b883",
  Nuxt: "#00dc82",
  Svelte: "#ff3e00",
  Angular: "#dd0031",
  Astro: "#ff5d01",
  Remix: "#3992ff",
  Flutter: "#02569B",
  Tauri: "#ffc131",
  Electron: "#47848f",
  Express: "#444444",
  NestJS: "#e0234e",
  Hono: "#e36002",
  Vite: "#646cff",
  Django: "#092e20",
  FastAPI: "#009688",
  Flask: "#000000",
  Laravel: "#ff2d20",
  // backend / services
  Firebase: "#ffca28",
  Supabase: "#3ecf8e",
  Vercel: "#000000",
  Netlify: "#00c7b7",
  "AWS Amplify": "#ff9900",
  "AWS SDK": "#ff9900",
  "Google Cloud": "#4285f4",
  Azure: "#0078d4",
  Cloudflare: "#f38020",
  "Cloudflare Workers": "#f38020",
  Appwrite: "#fd366e",
  Clerk: "#6c47ff",
  Auth0: "#eb5424",
  NextAuth: "#8a05ff",
  Stripe: "#635bff",
  Sentry: "#362d59",
  OpenAI: "#10a37f",
  Anthropic: "#d97757",
  Algolia: "#003dff",
  Sanity: "#f03e2f",
  Contentful: "#2478cc",
  Twilio: "#f22f46",
  SendGrid: "#1a82e2",
  GraphQL: "#e10098",
  Apollo: "#311c87",
  Heroku: "#430098",
  "Fly.io": "#8b5cf6",
  Railway: "#0b0d0e",
  Render: "#46e3b7",
  Docker: "#2496ed",
  "Docker Compose": "#2496ed",
  // databases
  Prisma: "#2d3748",
  Drizzle: "#c5f74f",
  MongoDB: "#47a248",
  PostgreSQL: "#4169e1",
  MySQL: "#4479a1",
  Redis: "#dc382d",
  SQLite: "#003b57",
  PlanetScale: "#000000",
  "Upstash Redis": "#00e9a3",
  Supabase_db: "#3ecf8e",
};

export function techColor(name: string, category: TechCategory): string {
  return TECH_COLORS[name] ?? CATEGORY_META[category].accent;
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
  technologies: TechHit[];
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
