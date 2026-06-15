# 言語判別 (Language Detector)

フォルダーを選ぶだけで、中で使われているプログラミング言語とフレームワークを
自動で判別して見える化するデスクトップアプリ。

## できること

- **フォルダー選択** — ネイティブのフォルダー選択ダイアログから対象を指定
- **言語判定** — 拡張子からファイルの言語を判定し、言語ごとにファイル数を集計
- **フレームワーク判定** — `package.json` / `Cargo.toml` / `pubspec.yaml` /
  `requirements.txt` / `pyproject.toml` / `composer.json` などの設定ファイルや
  構成ファイルを読んで、React・Next.js・Vue・Nuxt・Flutter・Django・FastAPI・
  Laravel・Tauri などを検出
- **結果表示** — ドーナツチャートで割合、棒グラフでファイル数、フレームワーク一覧、
  言語でフィルタできるファイル一覧を表示
- **除外** — `node_modules` / `.git` / `dist` / `target` などのディレクトリは自動で除外

## 技術構成

- フロントエンド: React 19 + Vite + TypeScript
- バックエンド: Tauri 2 (Rust) — フォルダーの再帰走査と判定ロジック
- グラフ: 依存ライブラリなしの自作 SVG ドーナツチャート / CSS 棒グラフ

主なファイル:

| ファイル | 役割 |
| --- | --- |
| `src-tauri/src/analyzer.rs` | 言語・フレームワーク判定のコアロジック |
| `src-tauri/src/lib.rs` | `analyze_folder` コマンドの登録 |
| `src/App.tsx` | 画面全体（選択・解析・結果表示） |
| `src/DonutChart.tsx` | SVG ドーナツチャート |
| `src/types.ts` | 型定義と言語カラー |

## 開発

```bash
npm install
npm run tauri dev      # 開発用にアプリを起動
npm run tauri build    # 配布用ビルド
```

テスト（判定ロジック）:

```bash
cd src-tauri && cargo test
```

## 拡張アイデア

- 除外フォルダーをユーザー設定できるようにする
- ファイルの中身まで見て拡張子だけでは曖昧なものを判定する
- 解析結果の CSV エクスポート
