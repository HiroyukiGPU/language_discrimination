// フォルダーを再帰的に走査して、使用されているプログラミング言語と
// フレームワークを判定するロジック。
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;

/// 言語ごとの集計結果。
#[derive(Serialize)]
pub struct LanguageStat {
    pub name: String,
    pub count: usize,
    pub percentage: f64,
}

/// 検出された技術（フレームワーク / バックエンド・サービス / データベース）。
#[derive(Serialize)]
pub struct TechHit {
    pub name: String,
    /// 種別: "framework" / "backend" / "database"。
    pub category: String,
    /// 何を根拠に判定したか（例: "package.json", "firebase.json"）。
    pub detected_by: String,
}

/// 個々のファイルの判定結果。
#[derive(Serialize)]
pub struct FileInfo {
    /// ルートからの相対パス。
    pub path: String,
    pub language: String,
    /// 判定方法: "extension" / "filename" / "shebang" / "content"。
    pub detection: String,
}

/// 言語をどう判定したか。
#[derive(Clone, Copy)]
enum Detection {
    /// 拡張子から判定。
    Extension,
    /// 特殊なファイル名から判定（Makefile など）。
    Filename,
    /// シェバン行（#!）から判定。
    Shebang,
    /// ファイルの中身（キーワードなど）から判定。
    Content,
}

impl Detection {
    fn as_str(self) -> &'static str {
        match self {
            Detection::Extension => "extension",
            Detection::Filename => "filename",
            Detection::Shebang => "shebang",
            Detection::Content => "content",
        }
    }
}

/// 解析全体の結果。
#[derive(Serialize)]
pub struct AnalysisResult {
    pub root: String,
    /// 走査したファイル総数（言語不明も含む）。
    pub total_files: usize,
    /// 言語を判定できたファイル数。
    pub counted_files: usize,
    pub languages: Vec<LanguageStat>,
    pub technologies: Vec<TechHit>,
    pub files: Vec<FileInfo>,
}

/// 技術の種別。
const CAT_FRAMEWORK: &str = "framework";
const CAT_BACKEND: &str = "backend";
const CAT_DATABASE: &str = "database";

/// 走査時に無視するディレクトリ名。
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "dist",
    "build",
    "out",
    "target",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "__pycache__",
    ".pytest_cache",
    "venv",
    ".venv",
    "env",
    ".idea",
    ".vscode",
    ".dart_tool",
    "Pods",
    "vendor",
    ".gradle",
];

/// 拡張子（小文字）からプログラミング言語名を返す。
/// 設定ファイルやデータ形式（json, yaml, toml, md など）は対象外。
fn language_for_extension(ext: &str) -> Option<&'static str> {
    let lang = match ext {
        "js" | "mjs" | "cjs" | "jsx" => "JavaScript",
        "ts" | "tsx" | "mts" | "cts" => "TypeScript",
        "py" | "pyw" => "Python",
        "dart" => "Dart",
        "swift" => "Swift",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "less" => "Less",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "php" => "PHP",
        "rb" => "Ruby",
        "rs" => "Rust",
        "go" => "Go",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "C++",
        "cs" => "C#",
        "vue" => "Vue",
        "svelte" => "Svelte",
        "sh" | "bash" | "zsh" => "Shell",
        "sql" => "SQL",
        "r" => "R",
        "lua" => "Lua",
        "scala" => "Scala",
        "ex" | "exs" => "Elixir",
        "pl" | "pm" => "Perl",
        _ => return None,
    };
    Some(lang)
}

/// 拡張子だけでは曖昧で、中身を見て判定し直す対象かどうか。
fn is_ambiguous_extension(ext: &str) -> bool {
    // .h は C / C++ / Objective-C のどれもありうる。
    matches!(ext, "h")
}

/// 拡張子を持たない（または .txt など）特定のファイル名から言語を判定する。
/// 例: Makefile, Gemfile, CMakeLists.txt など。
fn language_for_filename(name: &str) -> Option<&'static str> {
    let lang = match name {
        "Makefile" | "makefile" | "GNUmakefile" => "Makefile",
        "CMakeLists.txt" => "CMake",
        "Rakefile" | "Gemfile" | "Guardfile" | "Vagrantfile" | "Podfile" | "Brewfile" => "Ruby",
        "Pipfile" | "SConstruct" | "wscript" => "Python",
        _ => return None,
    };
    Some(lang)
}

/// ファイル先頭のシェバン行（#!）からインタプリタ言語を判定する。
fn language_from_shebang(first_line: &str) -> Option<&'static str> {
    let line = first_line.trim();
    if !line.starts_with("#!") {
        return None;
    }
    let lower = line.to_lowercase();
    // env 経由・直接指定の両方に対応するため、含まれる語で判定する。
    let lang = if lower.contains("python") {
        "Python"
    } else if lower.contains("node") || lower.contains("deno") || lower.contains("bun") {
        "JavaScript"
    } else if lower.contains("bash") || lower.contains("zsh") || lower.ends_with("/sh") || lower.contains("/sh ") {
        "Shell"
    } else if lower.contains("ruby") {
        "Ruby"
    } else if lower.contains("perl") {
        "Perl"
    } else if lower.contains("php") {
        "PHP"
    } else if lower.contains("lua") {
        "Lua"
    } else {
        return None;
    };
    Some(lang)
}

/// .h ファイルなど曖昧な拡張子について、中身から C++ か C かを判定する。
/// C++ 特有のトークンがあれば C++、なければ拡張子どおりの既定言語を返す。
fn refine_ambiguous(ext: &str, content: &str) -> &'static str {
    if ext == "h" {
        let cpp_markers = [
            "class ",
            "namespace ",
            "template",
            "std::",
            "::",
            "public:",
            "private:",
            "protected:",
            "#include <iostream>",
            "#include <vector>",
            "#include <string>",
        ];
        if cpp_markers.iter().any(|m| content.contains(m)) {
            return "C++";
        }
        return "C";
    }
    "C"
}

/// ファイル先頭を最大 `MAX_PEEK` バイトだけ読み込む（巨大ファイル対策）。
const MAX_PEEK: usize = 8 * 1024;

fn peek_content(path: &Path) -> Option<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; MAX_PEEK];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    // バイナリっぽい（NUL を含む）場合は対象外。
    if buf.contains(&0) {
        return None;
    }
    Some(String::from_utf8_lossy(&buf).into_owned())
}

/// 1 ファイルの言語と判定方法を決める。
/// 優先順位: 特殊ファイル名 → 拡張子（曖昧なら中身で補正） → シェバン。
fn detect_language(path: &Path) -> Option<(&'static str, Detection)> {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // 1. 特殊なファイル名（Makefile など）。
    if let Some(lang) = language_for_filename(file_name) {
        return Some((lang, Detection::Filename));
    }

    // 2. 拡張子。
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext = ext.to_lowercase();
        if let Some(lang) = language_for_extension(&ext) {
            // 曖昧な拡張子は中身を見て補正する。
            if is_ambiguous_extension(&ext) {
                if let Some(content) = peek_content(path) {
                    let refined = refine_ambiguous(&ext, &content);
                    let detection = if refined == lang {
                        Detection::Extension
                    } else {
                        Detection::Content
                    };
                    return Some((refined, detection));
                }
            }
            return Some((lang, Detection::Extension));
        }
        // 既知でない拡張子のファイルは、誤検出を避けるため中身判定しない。
        return None;
    }

    // 3. 拡張子なし → シェバン行で判定。
    let content = peek_content(path)?;
    let first_line = content.lines().next().unwrap_or("");
    language_from_shebang(first_line).map(|lang| (lang, Detection::Shebang))
}

/// 指定フォルダーを解析する。
pub fn analyze(root: &str) -> Result<AnalysisResult, String> {
    let root_path = Path::new(root);
    if !root_path.is_dir() {
        return Err(format!("フォルダーが見つかりません: {}", root));
    }

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut files: Vec<FileInfo> = Vec::new();
    let mut total_files = 0usize;
    let mut technologies: Vec<TechHit> = Vec::new();

    let walker = WalkDir::new(root_path).into_iter().filter_entry(|e| {
        // 除外ディレクトリは丸ごとスキップする。
        if e.file_type().is_dir() {
            if let Some(name) = e.file_name().to_str() {
                return !EXCLUDED_DIRS.contains(&name);
            }
        }
        true
    });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        total_files += 1;

        let path = entry.path();
        let file_name = entry.file_name().to_str().unwrap_or("");

        // 技術判定（設定ファイルを見る）。
        detect_technologies(file_name, path, &mut technologies);

        // 言語判定（拡張子 → ファイル名 → シェバン → 中身）。
        if let Some((lang, detection)) = detect_language(path) {
            *counts.entry(lang.to_string()).or_insert(0) += 1;
            let rel = path
                .strip_prefix(root_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            files.push(FileInfo {
                path: rel,
                language: lang.to_string(),
                detection: detection.as_str().to_string(),
            });
        }
    }

    let counted_files: usize = counts.values().sum();

    // 言語をファイル数の多い順に並べ、割合を計算する。
    let mut languages: Vec<LanguageStat> = counts
        .into_iter()
        .map(|(name, count)| LanguageStat {
            name,
            count,
            percentage: if counted_files > 0 {
                (count as f64 / counted_files as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();
    languages.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));

    // 技術の重複を除去する（同名は最初の 1 件だけ残す）。
    technologies.sort_by(|a, b| a.name.cmp(&b.name));
    technologies.dedup_by(|a, b| a.name == b.name);

    // ファイルも言語→パス順で並べる。
    files.sort_by(|a, b| a.language.cmp(&b.language).then(a.path.cmp(&b.path)));

    Ok(AnalysisResult {
        root: root.to_string(),
        total_files,
        counted_files,
        languages,
        technologies,
        files,
    })
}

/// 設定ファイルやフォルダー構成からフレームワークを判定する。
fn detect_frameworks(file_name: &str, path: &Path, out: &mut Vec<FrameworkHit>) {
    let mut push = |name: &str, by: &str| {
        out.push(FrameworkHit {
            name: name.to_string(),
            detected_by: by.to_string(),
        });
    };

    match file_name {
        "package.json" => detect_from_package_json(path, out),
        "pubspec.yaml" | "pubspec.yml" => push("Flutter", file_name),
        "Cargo.toml" => detect_from_cargo_toml(path, out),
        "requirements.txt" => detect_python_deps(path, out, file_name),
        "pyproject.toml" => detect_python_deps(path, out, file_name),
        "Gemfile" => push("Ruby on Rails?", file_name), // 大まかな目安
        "go.mod" => push("Go modules", file_name),
        "composer.json" => detect_from_composer_json(path, out),
        "nuxt.config.js" | "nuxt.config.ts" => push("Nuxt", file_name),
        "next.config.js" | "next.config.ts" | "next.config.mjs" => push("Next.js", file_name),
        "vite.config.js" | "vite.config.ts" => push("Vite", file_name),
        "svelte.config.js" => push("Svelte", file_name),
        "angular.json" => push("Angular", file_name),
        "tauri.conf.json" => push("Tauri", file_name),
        "Dockerfile" => push("Docker", file_name),
        _ => {}
    }
}

/// package.json の dependencies / devDependencies からフレームワークを判定。
fn detect_from_package_json(path: &Path, out: &mut Vec<FrameworkHit>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut deps: Vec<String> = Vec::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(obj) = json.get(key).and_then(|v| v.as_object()) {
            deps.extend(obj.keys().cloned());
        }
    }

    // 依存名 → フレームワーク名の対応。
    let map: &[(&str, &str)] = &[
        ("next", "Next.js"),
        ("nuxt", "Nuxt"),
        ("@angular/core", "Angular"),
        ("vue", "Vue"),
        ("svelte", "Svelte"),
        ("react-native", "React Native"),
        ("react", "React"),
        ("@remix-run/react", "Remix"),
        ("gatsby", "Gatsby"),
        ("astro", "Astro"),
        ("solid-js", "SolidJS"),
        ("vite", "Vite"),
        ("webpack", "Webpack"),
        ("express", "Express"),
        ("@nestjs/core", "NestJS"),
        ("koa", "Koa"),
        ("fastify", "Fastify"),
        ("electron", "Electron"),
        ("@tauri-apps/api", "Tauri"),
    ];

    for (dep, fw) in map {
        if deps.iter().any(|d| d == dep) {
            out.push(FrameworkHit {
                name: fw.to_string(),
                detected_by: "package.json".to_string(),
            });
        }
    }
}

/// Cargo.toml の dependencies からフレームワークを判定。
fn detect_from_cargo_toml(path: &Path, out: &mut Vec<FrameworkHit>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return,
    };

    let deps = value.get("dependencies").and_then(|d| d.as_table());
    let map: &[(&str, &str)] = &[
        ("tauri", "Tauri"),
        ("actix-web", "Actix Web"),
        ("axum", "Axum"),
        ("rocket", "Rocket"),
        ("warp", "Warp"),
        ("bevy", "Bevy"),
        ("leptos", "Leptos"),
        ("yew", "Yew"),
    ];
    if let Some(deps) = deps {
        for (dep, fw) in map {
            if deps.contains_key(*dep) {
                out.push(FrameworkHit {
                    name: fw.to_string(),
                    detected_by: "Cargo.toml".to_string(),
                });
            }
        }
    }
}

/// requirements.txt / pyproject.toml の中身から Python フレームワークを判定。
fn detect_python_deps(path: &Path, out: &mut Vec<FrameworkHit>, by: &str) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c.to_lowercase(),
        Err(_) => return,
    };
    let map: &[(&str, &str)] = &[
        ("django", "Django"),
        ("fastapi", "FastAPI"),
        ("flask", "Flask"),
        ("streamlit", "Streamlit"),
        ("pyramid", "Pyramid"),
        ("tornado", "Tornado"),
        ("scrapy", "Scrapy"),
    ];
    for (dep, fw) in map {
        if content.contains(dep) {
            out.push(FrameworkHit {
                name: fw.to_string(),
                detected_by: by.to_string(),
            });
        }
    }
}

/// composer.json の require から PHP フレームワークを判定。
fn detect_from_composer_json(path: &Path, out: &mut Vec<FrameworkHit>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };
    let mut deps: Vec<String> = Vec::new();
    for key in ["require", "require-dev"] {
        if let Some(obj) = json.get(key).and_then(|v| v.as_object()) {
            deps.extend(obj.keys().cloned());
        }
    }
    let map: &[(&str, &str)] = &[
        ("laravel/framework", "Laravel"),
        ("symfony/symfony", "Symfony"),
        ("symfony/framework-bundle", "Symfony"),
        ("cakephp/cakephp", "CakePHP"),
    ];
    for (dep, fw) in map {
        if deps.iter().any(|d| d == dep) {
            out.push(FrameworkHit {
                name: fw.to_string(),
                detected_by: "composer.json".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_mapping() {
        assert_eq!(language_for_extension("ts"), Some("TypeScript"));
        assert_eq!(language_for_extension("PY"), None); // 呼び出し側で小文字化する前提
        assert_eq!(language_for_extension("py"), Some("Python"));
        assert_eq!(language_for_extension("json"), None); // 設定/データは除外
        assert_eq!(language_for_extension("unknownext"), None);
    }

    #[test]
    fn shebang_detection() {
        assert_eq!(language_from_shebang("#!/usr/bin/env python3"), Some("Python"));
        assert_eq!(language_from_shebang("#!/bin/bash"), Some("Shell"));
        assert_eq!(language_from_shebang("#!/bin/sh"), Some("Shell"));
        assert_eq!(language_from_shebang("#!/usr/bin/env node"), Some("JavaScript"));
        assert_eq!(language_from_shebang("#!/usr/bin/ruby"), Some("Ruby"));
        assert_eq!(language_from_shebang("not a shebang"), None);
        assert_eq!(language_from_shebang("#!/unknown/thing"), None);
    }

    #[test]
    fn filename_detection() {
        assert_eq!(language_for_filename("Makefile"), Some("Makefile"));
        assert_eq!(language_for_filename("CMakeLists.txt"), Some("CMake"));
        assert_eq!(language_for_filename("Gemfile"), Some("Ruby"));
        assert_eq!(language_for_filename("random.txt"), None);
    }

    #[test]
    fn h_header_refinement() {
        assert_eq!(refine_ambiguous("h", "#include <stdio.h>\nint x;"), "C");
        assert_eq!(
            refine_ambiguous("h", "namespace foo { class Bar {}; }"),
            "C++"
        );
        assert_eq!(refine_ambiguous("h", "std::vector<int> v;"), "C++");
    }

    /// 一時ディレクトリに実ファイルを作り、内容判定が機能するか確認する。
    #[test]
    fn detects_extensionless_and_ambiguous_files() {
        let dir = std::env::temp_dir().join(format!("langtest_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        // 拡張子なし + python シェバン。
        std::fs::write(dir.join("deploy"), "#!/usr/bin/env python3\nprint('hi')\n").unwrap();
        // Makefile（ファイル名判定）。
        std::fs::write(dir.join("Makefile"), "all:\n\techo hi\n").unwrap();
        // C++ ヘッダー（.h だが中身は C++）。
        std::fs::write(dir.join("widget.h"), "#pragma once\nclass Widget {};\n").unwrap();
        // 純粋な C ヘッダー。
        std::fs::write(dir.join("util.h"), "#pragma once\nint add(int,int);\n").unwrap();

        let result = analyze(dir.to_str().unwrap()).expect("解析失敗");
        let by_path: std::collections::HashMap<_, _> = result
            .files
            .iter()
            .map(|f| (f.path.as_str(), (f.language.as_str(), f.detection.as_str())))
            .collect();

        assert_eq!(by_path.get("deploy"), Some(&("Python", "shebang")));
        assert_eq!(by_path.get("Makefile"), Some(&("Makefile", "filename")));
        assert_eq!(by_path.get("widget.h"), Some(&("C++", "content")));
        assert_eq!(by_path.get("util.h"), Some(&("C", "extension")));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// このプロジェクト自身（src-tauri の親）を解析し、
    /// TypeScript / Rust と Tauri / React / Vite が検出されることを確認する。
    #[test]
    fn analyzes_this_project() {
        let result = analyze("..").expect("解析に失敗");
        let langs: Vec<&str> = result.languages.iter().map(|l| l.name.as_str()).collect();
        assert!(langs.contains(&"TypeScript"), "languages={:?}", langs);
        assert!(langs.contains(&"Rust"), "languages={:?}", langs);

        let fws: Vec<&str> = result.frameworks.iter().map(|f| f.name.as_str()).collect();
        assert!(fws.contains(&"Tauri"), "frameworks={:?}", fws);
        assert!(fws.contains(&"React"), "frameworks={:?}", fws);
        assert!(fws.contains(&"Vite"), "frameworks={:?}", fws);

        // node_modules が除外されているか（あれば膨大になる）。
        assert!(!result.files.iter().any(|f| f.path.contains("node_modules")));

        // 割合の合計はほぼ 100%。
        let sum: f64 = result.languages.iter().map(|l| l.percentage).sum();
        assert!((sum - 100.0).abs() < 0.01 || result.counted_files == 0);
    }
}
