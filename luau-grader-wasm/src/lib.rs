use luau_grader_core::analyzer::analyze_graded;
use luau_grader_core::config::Tier;
use luau_grader_core::ruleset_config::RulesetConfig;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn grade_source(source: &str, tier_name: &str) -> Result<JsValue, JsValue> {
    let tier = tier_name.parse::<Tier>().unwrap_or(Tier::FrontPage);
    let config = RulesetConfig::default();

    match analyze_graded(source, tier, "memory.luau", &[], &config) {
        Ok(report) => {
            to_value(&report).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}
