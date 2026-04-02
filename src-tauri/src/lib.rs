use luau_grader_core::analyzer;
use luau_grader_core::config::Tier;
use luau_grader_core::fixer;
use luau_grader_core::grade::GradeReport;
use luau_grader_core::report::Report;
use luau_grader_core::ruleset_config::RulesetConfig;
use luau_grader_core::rulesets::{self, RuleInfo};
use serde::Deserialize;
use std::thread;

const STACK_SIZE: usize = 8 * 1024 * 1024;

#[derive(Deserialize, Clone)]
pub struct GradeRequest {
    pub source: String,
    pub tier: String,
    pub file_name: String,
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub config: Option<RulesetConfig>,
}

fn parse_tier(s: &str) -> Result<Tier, String> {
    s.parse().map_err(|e: String| e)
}

#[tauri::command]
fn grade_luau(request: GradeRequest) -> Result<GradeReport, String> {
    let req = request.clone();
    let handle = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let tier = parse_tier(&req.tier)?;
            let config = req.config.unwrap_or_default();
            analyzer::analyze_graded(
                &req.source, tier, &req.file_name,
                &req.disabled_rules, &config
            ).map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
    handle.join().map_err(|_| "analysis thread panicked".to_string())?
}

#[tauri::command]
fn grade_luau_basic(request: GradeRequest) -> Result<Report, String> {
    let tier = parse_tier(&request.tier)?;
    let config = request.config.unwrap_or_default();
    analyzer::analyze_with_config(&request.source, tier, &request.file_name, &request.disabled_rules, &config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn apply_fixes(request: GradeRequest) -> Result<fixer::FixReport, String> {
    let req = request.clone();
    let handle = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let tier = parse_tier(&req.tier)?;
            let config = req.config.unwrap_or_default();
            let report = analyzer::analyze_with_config(&req.source, tier, &req.file_name, &req.disabled_rules, &config)
                .map_err(|e| e.to_string())?;
            let rules = rulesets::rules_for_tier_with_config(tier, &req.disabled_rules, &config);
            Ok(fixer::apply_fixes(&req.source, &report.diagnostics, &rules))
        })
        .map_err(|e| e.to_string())?;
    handle.join().map_err(|_| "fix thread panicked".to_string())?
}

#[tauri::command]
fn list_rules() -> Vec<RuleInfo> {
    rulesets::list_all_rules()
}

#[tauri::command]
fn export_report(request: GradeRequest) -> Result<String, String> {
    let req = request.clone();
    let handle = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let tier = parse_tier(&req.tier)?;
            let config = req.config.unwrap_or_default();
            let report = analyzer::analyze_graded(
                &req.source, tier, &req.file_name,
                &req.disabled_rules, &config
            ).map_err(|e| e.to_string())?;
            Ok(luau_grader_core::export::export_markdown(&report))
        })
        .map_err(|e| e.to_string())?;
    handle.join().map_err(|_| "export thread panicked".to_string())?
}

#[tauri::command]
fn analyze_workspace(path: String) -> Result<Vec<luau_grader_core::workspace_rules::WorkspaceDiagnostic>, String> {
    let path = std::path::PathBuf::from(path);
    if !path.is_dir() {
        return Err("Selected path is not a directory.".to_string());
    }
    
    let handle = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let graph = luau_grader_core::workspace::analyze_workspace(&path).map_err(|e| e.to_string())?;
            let rules = luau_grader_core::workspace_rules::all_workspace_rules();
            let mut all_diags = Vec::new();
            
            for rule in rules {
                let diags = rule.analyze(&graph);
                all_diags.extend(diags);
            }
            Ok(all_diags)
        })
        .map_err(|e| e.to_string())?;
    
    handle.join().map_err(|_| "workspace analysis thread panicked".to_string())?
}

#[tauri::command]
fn pick_workspace_folder() -> Option<String> {
    rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().to_string())
}

fn get_themes_dir() -> std::path::PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(appdata).join("Luau Grader").join("themes")
}

fn bootstrap_themes(dir: &std::path::Path) {
    if !dir.exists() {
        let _ = std::fs::create_dir_all(dir);
    }

    let defaults: Vec<(&str, &str)> = vec![
        ("dracula.json", include_str!("../../themes/dracula.json")),
        ("tokyo_night.json", include_str!("../../themes/tokyo_night.json")),
        ("monochrome.json", include_str!("../../themes/monochrome.json")),
    ];

    for (name, content) in defaults {
        let path = dir.join(name);
        if !path.exists() {
            let _ = std::fs::write(&path, content);
        }
    }
}

#[tauri::command]
fn list_themes() -> Result<Vec<serde_json::Value>, String> {
    let dir = get_themes_dir();
    bootstrap_themes(&dir);

    let mut themes: Vec<serde_json::Value> = Vec::new();

    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if json.get("colors").is_some() && json.get("name").is_some() {
                        let mut theme = json;
                        if let Some(obj) = theme.as_object_mut() {
                            obj.insert("_filename".to_string(), serde_json::Value::String(
                                path.file_name().unwrap_or_default().to_string_lossy().to_string()
                            ));
                        }
                        themes.push(theme);
                    }
                }
            }
        }
    }

    themes.sort_by(|a, b| {
        let na = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        na.cmp(nb)
    });

    Ok(themes)
}

#[tauri::command]
fn open_themes_folder() -> Result<(), String> {
    let dir = get_themes_dir();
    bootstrap_themes(&dir);
    std::process::Command::new("explorer")
        .arg(dir.to_string_lossy().to_string())
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![grade_luau, grade_luau_basic, apply_fixes, list_rules, export_report, analyze_workspace, pick_workspace_folder, list_themes, open_themes_folder])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}