import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { DonutChart } from "./DonutChart";
import {
  AnalysisResult,
  colorFor,
  DETECTION_LABELS,
  CATEGORY_META,
  CATEGORY_ORDER,
  techColor,
} from "./types";
import "./App.css";

function App() {
  const [folder, setFolder] = useState<string | null>(null);
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileFilter, setFileFilter] = useState<string>("all");

  async function pickFolder() {
    setError(null);
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setFolder(selected);
      setResult(null);
    }
  }

  async function analyze() {
    if (!folder) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await invoke<AnalysisResult>("analyze_folder", { path: folder });
      setResult(res);
      setFileFilter("all");
    } catch (e) {
      setError(typeof e === "string" ? e : "解析中にエラーが発生しました");
    } finally {
      setLoading(false);
    }
  }

  const topLanguage = result?.languages[0];

  const filteredFiles = useMemo(() => {
    if (!result) return [];
    if (fileFilter === "all") return result.files;
    return result.files.filter((f) => f.language === fileFilter);
  }, [result, fileFilter]);

  return (
    <main className="app">
      <header className="hero">
        <h1>
          <span className="logo">{"</>"}</span> 言語判別
        </h1>
        <p className="subtitle">
          フォルダーを選ぶだけで、使われている言語とフレームワークを自動で見える化します。
        </p>
      </header>

      <section className="picker card">
        <div className="picker-row">
          <button className="btn btn-secondary" onClick={pickFolder}>
            📁 フォルダーを選択
          </button>
          <div className="path" title={folder ?? ""}>
            {folder ?? "フォルダーが選択されていません"}
          </div>
          <button className="btn btn-primary" onClick={analyze} disabled={!folder || loading}>
            {loading ? "解析中…" : "解析開始"}
          </button>
        </div>
        {error && <div className="error">⚠️ {error}</div>}
      </section>

      {loading && (
        <section className="card placeholder">
          <div className="spinner" />
          <p>フォルダーを走査しています…</p>
        </section>
      )}

      {result && !loading && (
        <>
          <section className="summary-grid">
            <div className="card chart-card">
              <h2>言語の割合</h2>
              {result.counted_files === 0 ? (
                <p className="muted">判定できる言語ファイルが見つかりませんでした。</p>
              ) : (
                <div className="chart-wrap">
                  <DonutChart languages={result.languages} />
                  <ul className="legend">
                    {result.languages.map((l) => (
                      <li key={l.name}>
                        <span className="dot" style={{ background: colorFor(l.name) }} />
                        <span className="legend-name">{l.name}</span>
                        <span className="legend-pct">{l.percentage.toFixed(1)}%</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>

            <div className="card stats-card">
              <h2>サマリー</h2>
              <div className="stat">
                <span className="stat-num">{result.total_files}</span>
                <span className="stat-label">走査ファイル総数</span>
              </div>
              <div className="stat">
                <span className="stat-num">{result.counted_files}</span>
                <span className="stat-label">言語判定したファイル</span>
              </div>
              <div className="stat">
                <span className="stat-num">{result.languages.length}</span>
                <span className="stat-label">検出した言語の種類</span>
              </div>
              {topLanguage && (
                <div className="top-lang">
                  最多:{" "}
                  <strong style={{ color: colorFor(topLanguage.name) }}>{topLanguage.name}</strong>
                  （{topLanguage.count}件）
                </div>
              )}
            </div>
          </section>

          <section className="card">
            <h2>言語ごとのファイル数</h2>
            <ul className="bars">
              {result.languages.map((l) => (
                <li key={l.name} className="bar-row">
                  <span className="bar-name">{l.name}</span>
                  <div className="bar-track">
                    <div
                      className="bar-fill"
                      style={{
                        width: `${l.percentage}%`,
                        background: colorFor(l.name),
                      }}
                    />
                  </div>
                  <span className="bar-count">{l.count}</span>
                </li>
              ))}
            </ul>
          </section>

          <section className="card">
            <h2>検出した技術スタック</h2>
            {result.technologies.length === 0 ? (
              <p className="muted">フレームワーク・サービスは検出されませんでした。</p>
            ) : (
              <div className="tech-groups">
                {CATEGORY_ORDER.map((cat) => {
                  const items = result.technologies.filter((t) => t.category === cat);
                  if (items.length === 0) return null;
                  const meta = CATEGORY_META[cat];
                  return (
                    <div className="tech-group" key={cat}>
                      <div className="tech-group-head">
                        <span className="tech-group-icon">{meta.icon}</span>
                        <span className="tech-group-title">{meta.label}</span>
                        <span className="tech-group-count">{items.length}</span>
                      </div>
                      <div className="tech-cards">
                        {items.map((t) => {
                          const color = techColor(t.name, t.category);
                          return (
                            <div
                              className="tech-card"
                              key={t.name}
                              style={{ ["--tech" as string]: color }}
                              title={`根拠: ${t.detected_by}`}
                            >
                              <span className="tech-mark" style={{ background: color }}>
                                {t.name.charAt(0).toUpperCase()}
                              </span>
                              <span className="tech-info">
                                <span className="tech-name">{t.name}</span>
                                <span className="tech-by">{t.detected_by}</span>
                              </span>
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </section>

          <section className="card">
            <div className="files-head">
              <h2>ファイル一覧</h2>
              <select value={fileFilter} onChange={(e) => setFileFilter(e.target.value)}>
                <option value="all">すべての言語 ({result.files.length})</option>
                {result.languages.map((l) => (
                  <option key={l.name} value={l.name}>
                    {l.name} ({l.count})
                  </option>
                ))}
              </select>
            </div>
            <div className="file-list">
              {filteredFiles.map((f) => (
                <div className="file-row" key={f.path}>
                  <span className="lang-badge" style={{ background: colorFor(f.language) }}>
                    {f.language}
                  </span>
                  <span className="file-path">{f.path}</span>
                  {f.detection !== "extension" && (
                    <span className="detect-tag" title={`${DETECTION_LABELS[f.detection]}から判定`}>
                      {DETECTION_LABELS[f.detection]}判定
                    </span>
                  )}
                </div>
              ))}
            </div>
          </section>
        </>
      )}
    </main>
  );
}

export default App;
