pub mod context;
pub mod walker;

use crate::config::Tier;
use crate::errors::GraderError;
use crate::report::Report;
use crate::ruleset_config::RulesetConfig;
use crate::rulesets;

pub fn analyze(source: &str, tier: Tier, file_name: &str, disabled_rules: &[String]) -> Result<Report, GraderError> {
    analyze_with_config(source, tier, file_name, disabled_rules, &RulesetConfig::default())
}

pub fn analyze_with_config(
    source: &str,
    tier: Tier,
    file_name: &str,
    disabled_rules: &[String],
    config: &RulesetConfig,
) -> Result<Report, GraderError> {
    let ast = full_moon::parse(source)
        .map_err(|errors| {
            let msg = errors.iter().map(|e| format!("{e}")).collect::<Vec<_>>().join("; ");
            GraderError::Parse(msg)
        })?;

    let rules = rulesets::rules_for_tier_with_config(tier, disabled_rules, config);
    let mut ctx = context::AnalysisContext::new(source.to_string());
    let mut report = Report::new(file_name.to_string(), tier.to_string());

    let walker = walker::GraderWalker::new(&rules, &mut ctx);
    report.merge(walker.walk(&ast));

    for rule in &rules {
        report.merge(rule.finalize(&ctx));
    }

    for diag in &mut report.diagnostics {
        if let Some(rule) = rules.iter().find(|r| r.id() == diag.rule_id) {
            diag.fixable = rule.is_fixable();
        }
    }

    if !config.severity_overrides.is_empty() {
        report.apply_severity_overrides(&config.severity_overrides);
    }

    Ok(report)
}