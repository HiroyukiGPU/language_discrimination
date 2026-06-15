mod analyzer;

use analyzer::AnalysisResult;

/// フロントエンドから呼ばれるコマンド。指定フォルダーを解析する。
#[tauri::command]
fn analyze_folder(path: String) -> Result<AnalysisResult, String> {
    analyzer::analyze(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![analyze_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
