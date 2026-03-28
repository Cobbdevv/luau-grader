use luau_grader_core::analyzer;
use luau_grader_core::config::Tier;
use luau_grader_core::fixer;
use luau_grader_core::report::Report;
use luau_grader_core::ruleset_config::RulesetConfig;
use luau_grader_core::rulesets::{self, RuleInfo};
use serde::Deserialize;

#[derive(Deserialize)]
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
fn grade_luau(request: GradeRequest) -> Result<Report, String> {
    let tier = parse_tier(&request.tier)?;
    let config = request.config.unwrap_or_default();
    analyzer::analyze_with_config(&request.source, tier, &request.file_name, &request.disabled_rules, &config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn apply_fixes(request: GradeRequest) -> Result<fixer::FixReport, String> {
    let tier = parse_tier(&request.tier)?;
    let config = request.config.unwrap_or_default();
    let report = analyzer::analyze_with_config(&request.source, tier, &request.file_name, &request.disabled_rules, &config)
        .map_err(|e| e.to_string())?;
    let rules = rulesets::rules_for_tier_with_config(tier, &request.disabled_rules, &config);
    Ok(fixer::apply_fixes(&request.source, &report.diagnostics, &rules))
}

#[tauri::command]
fn list_rules() -> Vec<RuleInfo> {
    rulesets::list_all_rules()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![grade_luau, apply_fixes, list_rules])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}