use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::GraderError;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RulesetConfig {
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub severity_overrides: HashMap<String, String>,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub custom_rules: Vec<CustomRuleConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomRuleConfig {
    pub id: String,
    pub description: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_tier")]
    pub tier: String,
    pub pattern: PatternConfig,
    pub message: String,
    #[serde(default)]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PatternConfig {
    #[serde(rename = "function_call")]
    FunctionCall { name: String },
    #[serde(rename = "method_call")]
    MethodCall { name: String },
}

fn default_severity() -> String { "Warning".to_string() }
fn default_category() -> String { "Custom".to_string() }
fn default_tier() -> String { "Beginner".to_string() }

impl RulesetConfig {
    pub fn load(path: &Path) -> Result<Self, GraderError> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| GraderError::Config(format!("invalid .luaugraderrc: {e}")))
    }

    pub fn find_in_ancestors(start: &Path) -> Option<Self> {
        let mut dir = if start.is_file() {
            start.parent()?.to_path_buf()
        } else {
            start.to_path_buf()
        };
        loop {
            let config_path = dir.join(".luaugraderrc");
            if config_path.exists() {
                return Self::load(&config_path).ok();
            }
            if !dir.pop() { break; }
        }
        None
    }

    pub fn get_param<T: serde::de::DeserializeOwned>(&self, rule_id: &str, key: &str) -> Option<T> {
        let rule_params = self.params.get(rule_id)?;
        let value = rule_params.get(key)?;
        serde_json::from_value(value.clone()).ok()
    }
}
