use luau_grader_core::analyzer;
use luau_grader_core::config::Tier;
use luau_grader_core::fixer;
use luau_grader_core::ruleset_config::RulesetConfig;
use luau_grader_core::rulesets;

fn grade(source: &str, tier: Tier) -> Vec<String> {
    let report = analyzer::analyze(source, tier, "test.luau", &[]).unwrap();
    report.diagnostics.iter().map(|d| d.rule_id.clone()).collect()
}

fn grade_filtered(source: &str, tier: Tier, disabled: &[&str]) -> Vec<String> {
    let disabled: Vec<String> = disabled.iter().map(|s| s.to_string()).collect();
    let report = analyzer::analyze(source, tier, "test.luau", &disabled).unwrap();
    report.diagnostics.iter().map(|d| d.rule_id.clone()).collect()
}

#[test]
fn b001_deprecated_wait() {
    let ids = grade("wait(1)\nprint('hello')\n", Tier::Beginner);
    assert!(ids.contains(&"B001".to_string()));
}

#[test]
fn b001_clean_code() {
    let ids = grade("task.wait(1)\n", Tier::Beginner);
    assert!(!ids.contains(&"B001".to_string()));
}

#[test]
fn b002_deprecated_spawn() {
    let ids = grade("spawn(function() end)\n", Tier::Beginner);
    assert!(ids.contains(&"B002".to_string()));
}

#[test]
fn b003_deprecated_delay() {
    let ids = grade("delay(1, function() end)\n", Tier::Beginner);
    assert!(ids.contains(&"B003".to_string()));
}

#[test]
fn b004_invoke_client() {
    let ids = grade("remote:InvokeClient(player, data)\n", Tier::Beginner);
    assert!(ids.contains(&"B004".to_string()));
}

#[test]
fn b005_waitforchild_no_timeout() {
    let ids = grade("script:WaitForChild(\"Part\")\n", Tier::Beginner);
    assert!(ids.contains(&"B005".to_string()));
}

#[test]
fn b005_waitforchild_with_timeout() {
    let ids = grade("script:WaitForChild(\"Part\", 5)\n", Tier::Beginner);
    assert!(!ids.contains(&"B005".to_string()));
}

#[test]
fn i001_function_too_long() {
    let mut lines = vec!["local function big()".to_string()];
    for i in 0..55 { lines.push(format!("    print({})", i)); }
    lines.push("end".to_string());
    let source = lines.join("\n");
    let ids = grade(&source, Tier::Intermediate);
    assert!(ids.contains(&"I001".to_string()));
}

#[test]
fn i001_short_function() {
    let source = "local function small()\n    print(1)\nend\n";
    let ids = grade(source, Tier::Intermediate);
    assert!(!ids.contains(&"I001".to_string()));
}

#[test]
fn i002_single_letter_var() {
    let ids = grade("local x = 5\n", Tier::Intermediate);
    assert!(ids.contains(&"I002".to_string()));
}

#[test]
fn i002_loop_index_allowed() {
    let ids = grade("local i = 5\n", Tier::Intermediate);
    assert!(!ids.contains(&"I002".to_string()));
}

#[test]
fn i003_getservice_in_loop() {
    let source = "while true do\n    local svc = game:GetService(\"Players\")\nend\n";
    let ids = grade(source, Tier::Intermediate);
    assert!(ids.contains(&"I003".to_string()));
}

#[test]
fn i003_getservice_outside_loop() {
    let source = "local Players = game:GetService(\"Players\")\n";
    let ids = grade(source, Tier::Intermediate);
    assert!(!ids.contains(&"I003".to_string()));
}

#[test]
fn a001_instance_new_in_loop() {
    let source = "while true do\n    local p = Instance.new(\"Part\")\nend\n";
    let ids = grade(source, Tier::Advanced);
    assert!(ids.contains(&"A001".to_string()));
}

#[test]
fn a001_instance_new_outside_loop() {
    let source = "local p = Instance.new(\"Part\")\n";
    let ids = grade(source, Tier::Advanced);
    assert!(!ids.contains(&"A001".to_string()));
}

#[test]
fn a002_connect_without_store() {
    let source = "humanoid.Died:Connect(function() end)\n";
    let ids = grade(source, Tier::Advanced);
    assert!(ids.contains(&"A002".to_string()));
}

#[test]
fn a002_connect_stored() {
    let source = "local conn = humanoid.Died:Connect(function() end)\n";
    let ids = grade(source, Tier::Advanced);
    assert!(!ids.contains(&"A002".to_string()));
}

#[test]
fn a003_string_concat_in_loop() {
    let source = "for i = 1, 10 do\n    local s = \"a\" .. \"b\"\nend\n";
    let ids = grade(source, Tier::Advanced);
    assert!(ids.contains(&"A003".to_string()));
}

#[test]
fn a004_setasync() {
    let source = "dataStore:SetAsync(\"key\", data)\n";
    let ids = grade(source, Tier::Advanced);
    assert!(ids.contains(&"A004".to_string()));
}

#[test]
fn f001_no_strict_mode() {
    let ids = grade("local x = 1\n", Tier::FrontPage);
    assert!(ids.contains(&"F001".to_string()));
}

#[test]
fn f001_has_strict_mode() {
    let ids = grade("--!strict\nlocal x = 1\n", Tier::FrontPage);
    assert!(!ids.contains(&"F001".to_string()));
}

#[test]
fn f003_require_in_loop() {
    let source = "while true do\n    local m = require(script.Module)\nend\n";
    let ids = grade(source, Tier::FrontPage);
    assert!(ids.contains(&"F003".to_string()));
}

#[test]
fn f002_parent_nil_without_destroy() {
    let ids = grade("--!strict\npart.Parent = nil\n", Tier::FrontPage);
    assert!(ids.contains(&"F002".to_string()));
}

#[test]
fn f002_using_destroy() {
    let ids = grade("--!strict\npart:Destroy()\n", Tier::FrontPage);
    assert!(!ids.contains(&"F002".to_string()));
}

#[test]
fn intermediate_includes_beginner() {
    let ids = grade("wait(1)\n", Tier::Intermediate);
    assert!(ids.contains(&"B001".to_string()));
}

#[test]
fn frontpage_includes_all() {
    let source = "wait(1)\nhumanoid.Died:Connect(function() end)\n";
    let ids = grade(source, Tier::FrontPage);
    assert!(ids.contains(&"B001".to_string()));
    assert!(ids.contains(&"A002".to_string()));
    assert!(ids.contains(&"F001".to_string()));
}

#[test]
fn disabled_rules_are_skipped() {
    let ids = grade_filtered("wait(1)\n", Tier::Beginner, &["B001"]);
    assert!(!ids.contains(&"B001".to_string()));
}

#[test]
fn parse_error_returns_err() {
    let result = analyzer::analyze("function(", Tier::Beginner, "test.luau", &[]);
    assert!(result.is_err());
}

#[test]
fn empty_source_no_panic() {
    let result = analyzer::analyze("", Tier::FrontPage, "test.luau", &[]);
    assert!(result.is_ok());
}

#[test]
fn comments_only_source() {
    let report = analyzer::analyze("-- just a comment\n", Tier::FrontPage, "test.luau", &[]).unwrap();
    assert!(report.diagnostics.iter().any(|d| d.rule_id == "F001"));
}

#[test]
fn fix_deprecated_wait() {
    let source = "wait(1)\n";
    let report = analyzer::analyze(source, Tier::Beginner, "test.luau", &[]).unwrap();
    let rules = rulesets::rules_for_tier(Tier::Beginner, &[]);
    let fix_report = fixer::apply_fixes(source, &report.diagnostics, &rules);
    assert!(fix_report.fixed_source.contains("task.wait(1)"));
    assert!(!fix_report.applied.is_empty());
}

#[test]
fn fix_missing_strict() {
    let source = "local x = 1\n";
    let report = analyzer::analyze(source, Tier::FrontPage, "test.luau", &[]).unwrap();
    let rules = rulesets::rules_for_tier(Tier::FrontPage, &[]);
    let fix_report = fixer::apply_fixes(source, &report.diagnostics, &rules);
    assert!(fix_report.fixed_source.starts_with("--!strict"));
}

#[test]
fn fix_setasync() {
    let source = "dataStore:SetAsync(\"key\", data)\n";
    let report = analyzer::analyze(source, Tier::Advanced, "test.luau", &[]).unwrap();
    let rules = rulesets::rules_for_tier(Tier::Advanced, &[]);
    let fix_report = fixer::apply_fixes(source, &report.diagnostics, &rules);
    assert!(fix_report.fixed_source.contains("UpdateAsync"));
}

#[test]
fn config_custom_function_rule() {
    let config_json = r#"{
        "custom_rules": [{
            "id": "C001",
            "description": "No print in production",
            "severity": "Warning",
            "category": "Custom",
            "tier": "Beginner",
            "pattern": { "type": "function_call", "name": "print" },
            "message": "remove print() before shipping"
        }]
    }"#;
    let config: RulesetConfig = serde_json::from_str(config_json).unwrap();
    let report = analyzer::analyze_with_config("print(\"hello\")\n", Tier::Beginner, "test.luau", &[], &config).unwrap();
    assert!(report.diagnostics.iter().any(|d| d.rule_id == "C001"));
}

#[test]
fn config_custom_method_rule() {
    let config_json = r#"{
        "custom_rules": [{
            "id": "C002",
            "description": "No Clone calls",
            "severity": "Error",
            "category": "Custom",
            "tier": "Beginner",
            "pattern": { "type": "method_call", "name": "Clone" },
            "message": "avoid Clone — use explicit construction"
        }]
    }"#;
    let config: RulesetConfig = serde_json::from_str(config_json).unwrap();
    let report = analyzer::analyze_with_config("part:Clone()\n", Tier::Beginner, "test.luau", &[], &config).unwrap();
    assert!(report.diagnostics.iter().any(|d| d.rule_id == "C002"));
}

#[test]
fn config_param_max_lines() {
    let config_json = r#"{ "params": { "I001": { "max_lines": 10 } } }"#;
    let config: RulesetConfig = serde_json::from_str(config_json).unwrap();
    let mut lines = vec!["local function f()".to_string()];
    for i in 0..15 { lines.push(format!("    print({})", i)); }
    lines.push("end".to_string());
    let source = lines.join("\n");
    let report = analyzer::analyze_with_config(&source, Tier::Intermediate, "test.luau", &[], &config).unwrap();
    assert!(report.diagnostics.iter().any(|d| d.rule_id == "I001"));
}

#[test]
fn config_severity_override() {
    let config_json = r#"{ "severity_overrides": { "B001": "Error" } }"#;
    let config: RulesetConfig = serde_json::from_str(config_json).unwrap();
    let report = analyzer::analyze_with_config("wait(1)\n", Tier::Beginner, "test.luau", &[], &config).unwrap();
    let diag = report.diagnostics.iter().find(|d| d.rule_id == "B001").unwrap();
    assert_eq!(diag.severity, luau_grader_core::report::Severity::Error);
}

#[test]
fn fixable_flag_set_on_diagnostics() {
    let report = analyzer::analyze("wait(1)\n", Tier::Beginner, "test.luau", &[]).unwrap();
    let diag = report.diagnostics.iter().find(|d| d.rule_id == "B001").unwrap();
    assert!(diag.fixable);
}

#[test]
fn non_fixable_flag_set() {
    let report = analyzer::analyze("remote:InvokeClient(player, data)\n", Tier::Beginner, "test.luau", &[]).unwrap();
    let diag = report.diagnostics.iter().find(|d| d.rule_id == "B004").unwrap();
    assert!(!diag.fixable);
}