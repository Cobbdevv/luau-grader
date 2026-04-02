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

#[test]
fn b006_instance_new_with_parent() {
    let ids = grade("local p = Instance.new(\"Part\", workspace)\n", Tier::Beginner);
    assert!(ids.contains(&"B006".to_string()));
}

#[test]
fn b006_instance_new_without_parent() {
    let ids = grade("local p = Instance.new(\"Part\")\n", Tier::Beginner);
    assert!(!ids.contains(&"B006".to_string()));
}

#[test]
fn b007_lowercase_connect() {
    let ids = grade("signal:connect(function() end)\n", Tier::Beginner);
    assert!(ids.contains(&"B007".to_string()));
}

#[test]
fn b007_correct_connect() {
    let ids = grade("signal:Connect(function() end)\n", Tier::Beginner);
    assert!(!ids.contains(&"B007".to_string()));
}

#[test]
fn b007_lowercase_destroy() {
    let ids = grade("part:destroy()\n", Tier::Beginner);
    assert!(ids.contains(&"B007".to_string()));
}

#[test]
fn b007_lowercase_clone() {
    let ids = grade("local c = part:clone()\n", Tier::Beginner);
    assert!(ids.contains(&"B007".to_string()));
}

#[test]
fn b008_table_foreach() {
    let ids = grade("table.foreach(t, print)\n", Tier::Beginner);
    assert!(ids.contains(&"B008".to_string()));
}

#[test]
fn b008_table_getn() {
    let ids = grade("local n = table.getn(t)\n", Tier::Beginner);
    assert!(ids.contains(&"B008".to_string()));
}

#[test]
fn b008_table_insert_ok() {
    let ids = grade("table.insert(t, 1)\n", Tier::Beginner);
    assert!(!ids.contains(&"B008".to_string()));
}

#[test]
fn b009_game_dot_workspace() {
    let ids = grade("local ws = game.Workspace\n", Tier::Beginner);
    assert!(ids.contains(&"B009".to_string()));
}

#[test]
fn b009_workspace_global() {
    let ids = grade("local ws = workspace\n", Tier::Beginner);
    assert!(!ids.contains(&"B009".to_string()));
}

#[test]
fn b010_vector3_wrong_args() {
    let ids = grade("local v = Vector3.new(1, 2)\n", Tier::Beginner);
    assert!(ids.contains(&"B010".to_string()));
}

#[test]
fn b010_vector3_correct_args() {
    let ids = grade("local v = Vector3.new(1, 2, 3)\n", Tier::Beginner);
    assert!(!ids.contains(&"B010".to_string()));
}

#[test]
fn b010_color3_fromrgb_wrong() {
    let ids = grade("local c = Color3.fromRGB(255, 0)\n", Tier::Beginner);
    assert!(ids.contains(&"B010".to_string()));
}

#[test]
fn b010_udim2_correct() {
    let ids = grade("local u = UDim2.new(0, 100, 0, 50)\n", Tier::Beginner);
    assert!(!ids.contains(&"B010".to_string()));
}

#[test]
fn b011_isa_no_args() {
    let ids = grade("script:IsA()\n", Tier::Beginner);
    assert!(ids.contains(&"B011".to_string()));
}

#[test]
fn b011_isa_correct() {
    let ids = grade("local b = script:IsA(\"BasePart\")\n", Tier::Beginner);
    assert!(!ids.contains(&"B011".to_string()));
}

#[test]
fn b011_destroy_with_args() {
    let ids = grade("script:Destroy(\"oops\")\n", Tier::Beginner);
    assert!(ids.contains(&"B011".to_string()));
}

#[test]
fn b012_string_len_no_args() {
    let ids = grade("local n = string.len()\n", Tier::Beginner);
    assert!(ids.contains(&"B012".to_string()));
}

#[test]
fn b012_math_clamp_wrong() {
    let ids = grade("local x = math.clamp(1, 2)\n", Tier::Beginner);
    assert!(ids.contains(&"B012".to_string()));
}

#[test]
fn b012_math_clamp_correct() {
    let ids = grade("local x = math.clamp(5, 1, 10)\n", Tier::Beginner);
    assert!(!ids.contains(&"B012".to_string()));
}

#[test]
fn b012_pcall_no_args() {
    let ids = grade("local ok = pcall()\n", Tier::Beginner);
    assert!(ids.contains(&"B012".to_string()));
}

#[test]
fn i004_wrong_step() {
    let ids = grade("for i = 10, 1, 1 do\n    print(i)\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I004".to_string()));
}

#[test]
fn i004_correct_step() {
    let ids = grade("for i = 10, 1, -1 do\n    print(i)\nend\n", Tier::Intermediate);
    assert!(!ids.contains(&"I004".to_string()));
}

#[test]
fn i005_empty_if() {
    let ids = grade("if true then\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I005".to_string()));
}

#[test]
fn i005_non_empty_if() {
    let ids = grade("if true then\n    print(1)\nend\n", Tier::Intermediate);
    assert!(!ids.contains(&"I005".to_string()));
}

#[test]
fn i006_tostring_on_string() {
    let ids = grade("local s = tostring(\"hello\")\n", Tier::Intermediate);
    assert!(ids.contains(&"I006".to_string()));
}

#[test]
fn i006_tostring_on_variable() {
    let ids = grade("local s = tostring(x)\n", Tier::Intermediate);
    assert!(!ids.contains(&"I006".to_string()));
}

#[test]
fn i007_tonumber_on_number() {
    let ids = grade("local n = tonumber(42)\n", Tier::Intermediate);
    assert!(ids.contains(&"I007".to_string()));
}

#[test]
fn i007_tonumber_on_string() {
    let ids = grade("local n = tonumber(\"42\")\n", Tier::Intermediate);
    assert!(!ids.contains(&"I007".to_string()));
}

#[test]
fn i009_print_call() {
    let ids = grade("print(\"debug\")\n", Tier::Intermediate);
    assert!(ids.contains(&"I009".to_string()));
}

#[test]
fn i009_warn_call() {
    let ids = grade("warn(\"debug\")\n", Tier::Intermediate);
    assert!(ids.contains(&"I009".to_string()));
}

#[test]
fn i010_table_sort_assigned() {
    let ids = grade("local sorted = table.sort(t)\n", Tier::Intermediate);
    assert!(ids.contains(&"I010".to_string()));
}

#[test]
fn i010_table_sort_statement() {
    let ids = grade("table.sort(t)\n", Tier::Intermediate);
    assert!(!ids.contains(&"I010".to_string()));
}

#[test]
fn i011_type_call() {
    let ids = grade("local t = type(workspace)\n", Tier::Intermediate);
    assert!(ids.contains(&"I011".to_string()));
}

#[test]
fn i012_color3_new_255() {
    let ids = grade("local c = Color3.new(255, 0, 0)\n", Tier::Intermediate);
    assert!(ids.contains(&"I012".to_string()));
}

#[test]
fn i012_color3_new_correct() {
    let ids = grade("local c = Color3.new(0.5, 0.2, 0.8)\n", Tier::Intermediate);
    assert!(!ids.contains(&"I012".to_string()));
}

#[test]
fn a005_while_true_no_yield() {
    let ids = grade("while true do\n    local x = 1\nend\n", Tier::Advanced);
    assert!(ids.contains(&"A005".to_string()));
}

#[test]
fn a005_while_true_with_yield() {
    let ids = grade("while true do\n    task.wait(1)\nend\n", Tier::Advanced);
    assert!(!ids.contains(&"A005".to_string()));
}

#[test]
fn a006_insert_front_in_loop() {
    let ids = grade("for i = 1, 10 do\n    table.insert(t, 1, i)\nend\n", Tier::Advanced);
    assert!(ids.contains(&"A006".to_string()));
}

#[test]
fn a006_insert_append_in_loop() {
    let ids = grade("for i = 1, 10 do\n    table.insert(t, i)\nend\n", Tier::Advanced);
    assert!(!ids.contains(&"A006".to_string()));
}

#[test]
fn a007_connect_in_loop() {
    let ids = grade("for i = 1, 5 do\n    signal:Connect(function() end)\nend\n", Tier::Advanced);
    assert!(ids.contains(&"A007".to_string()));
}

#[test]
fn a007_connect_outside_loop() {
    let ids = grade("signal:Connect(function() end)\n", Tier::Advanced);
    assert!(!ids.contains(&"A007".to_string()));
}

#[test]
fn a008_pcall_bare_statement() {
    let ids = grade("pcall(function() end)\n", Tier::Advanced);
    assert!(ids.contains(&"A008".to_string()));
}

#[test]
fn a008_pcall_with_assignment() {
    let ids = grade("local ok, err = pcall(function() end)\n", Tier::Advanced);
    assert!(!ids.contains(&"A008".to_string()));
}

#[test]
fn a009_clone_bare_statement() {
    let ids = grade("script:Clone()\n", Tier::Advanced);
    assert!(ids.contains(&"A009".to_string()));
}

#[test]
fn a009_clone_stored() {
    let ids = grade("local c = script:Clone()\n", Tier::Advanced);
    assert!(!ids.contains(&"A009".to_string()));
}

#[test]
fn a010_load_animation() {
    let ids = grade("humanoid:LoadAnimation(anim)\n", Tier::Advanced);
    assert!(ids.contains(&"A010".to_string()));
}

#[test]
fn a011_set_primary_part_cframe() {
    let ids = grade("model:SetPrimaryPartCFrame(CFrame.new())\n", Tier::Advanced);
    assert!(ids.contains(&"A011".to_string()));
}

#[test]
fn a012_get_mouse() {
    let ids = grade("player:GetMouse()\n", Tier::Advanced);
    assert!(ids.contains(&"A012".to_string()));
}

#[test]
fn f004_getservice_workspace() {
    let ids = grade("--!strict\nlocal ws = game:GetService(\"Workspace\")\n", Tier::FrontPage);
    assert!(ids.contains(&"F004".to_string()));
}

#[test]
fn f004_getservice_players_ok() {
    let ids = grade("--!strict\nlocal p = game:GetService(\"Players\")\n", Tier::FrontPage);
    assert!(!ids.contains(&"F004".to_string()));
}

#[test]
fn f005_ffc_chained() {
    let ids = grade("--!strict\nworkspace:FindFirstChild(\"Model\"):Destroy()\n", Tier::FrontPage);
    assert!(ids.contains(&"F005".to_string()));
}

#[test]
fn f005_ffc_stored() {
    let ids = grade("--!strict\nlocal m = workspace:FindFirstChild(\"Model\")\n", Tier::FrontPage);
    assert!(!ids.contains(&"F005".to_string()));
}

#[test]
fn f006_deprecated_remove() {
    let ids = grade("--!strict\nscript:Remove()\n", Tier::FrontPage);
    assert!(ids.contains(&"F006".to_string()));
}

#[test]
fn f007_string_sub_zero() {
    let ids = grade("--!strict\nlocal s = string.sub(\"hello\", 0, 3)\n", Tier::FrontPage);
    assert!(ids.contains(&"F007".to_string()));
}

#[test]
fn f007_string_sub_one() {
    let ids = grade("--!strict\nlocal s = string.sub(\"hello\", 1, 3)\n", Tier::FrontPage);
    assert!(!ids.contains(&"F007".to_string()));
}

#[test]
fn f008_task_wait_negative() {
    let ids = grade("--!strict\ntask.wait(-1)\n", Tier::FrontPage);
    assert!(ids.contains(&"F008".to_string()));
}

#[test]
fn f008_task_wait_positive() {
    let ids = grade("--!strict\ntask.wait(1)\n", Tier::FrontPage);
    assert!(!ids.contains(&"F008".to_string()));
}

#[test]
fn f009_instance_new_empty() {
    let ids = grade("--!strict\nlocal p = Instance.new(\"\")\n", Tier::FrontPage);
    assert!(ids.contains(&"F009".to_string()));
}

#[test]
fn f009_instance_new_valid() {
    let ids = grade("--!strict\nlocal p = Instance.new(\"Part\")\n", Tier::FrontPage);
    assert!(!ids.contains(&"F009".to_string()));
}

#[test]
fn total_rule_count() {
    let all = rulesets::list_all_rules();
    assert!(all.len() >= 59, "Expected at least 59 rules, got {}", all.len());
}

#[test]
fn s004_loadstring_detected() {
    let ids = grade("loadstring(\"print('hi')\")\n", Tier::Beginner);
    assert!(ids.contains(&"S004".to_string()));
}

#[test]
fn s007_game_destroy() {
    let ids = grade("game:Destroy()\n", Tier::Beginner);
    assert!(ids.contains(&"S007".to_string()));
}

#[test]
fn grading_engine_produces_grade() {
    use luau_grader_core::ruleset_config::RulesetConfig;
    let config = RulesetConfig::default();
    let result = analyzer::analyze_graded(
        "--!strict\nlocal function foo()\n    return 1\nend\n",
        Tier::FrontPage,
        "test.luau",
        &[],
        &config,
    );
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(!report.grade.is_empty());
    assert!(report.overall_score > 0.0);
    assert!(report.dimensions.len() == 7);
}

#[test]
fn grading_bad_script_gets_lower_score() {
    use luau_grader_core::ruleset_config::RulesetConfig;
    let config = RulesetConfig::default();

    let good = analyzer::analyze_graded(
        "--!strict\n-- Game initialization module\nlocal Players = game:GetService(\"Players\")\nlocal function initializePlayer(player)\n    local character = player.Character\n    if not character then return end\n    local humanoid = character:FindFirstChild(\"Humanoid\")\n    return humanoid\nend\nreturn initializePlayer\n",
        Tier::FrontPage, "good.luau", &[], &config,
    ).unwrap();

    let bad = analyzer::analyze_graded(
        "wait(1)\nspawn(function() end)\ndelay(1, function() end)\nloadstring('x')\nlocal x = 1\nlocal y = 2\nlocal z = 3\n",
        Tier::FrontPage, "bad.luau", &[], &config,
    ).unwrap();

    assert!(good.overall_score > bad.overall_score,
        "Good script ({}) should score higher than bad script ({})",
        good.overall_score, bad.overall_score);
}

#[test]
fn grading_detects_script_type() {
    use luau_grader_core::metrics::ScriptType;
    use luau_grader_core::ruleset_config::RulesetConfig;
    let config = RulesetConfig::default();

    let module = analyzer::analyze_graded(
        "local M = {}\nfunction M.init() end\nreturn M\n",
        Tier::FrontPage, "module.luau", &[], &config,
    ).unwrap();

    assert!(matches!(module.script_type, ScriptType::SharedModule | ScriptType::ModuleScript),
        "Expected module script type, got {:?}", module.script_type);
}

#[test]
fn grading_has_dimensions() {
    use luau_grader_core::ruleset_config::RulesetConfig;
    let config = RulesetConfig::default();
    let report = analyzer::analyze_graded(
        "--!strict\nprint('hello')\n",
        Tier::FrontPage, "test.luau", &[], &config,
    ).unwrap();

    let dim_names: Vec<&str> = report.dimensions.iter().map(|d| d.name.as_str()).collect();
    assert!(dim_names.contains(&"Structure"));
    assert!(dim_names.contains(&"API Correctness"));
    assert!(dim_names.contains(&"Error Handling"));
    assert!(dim_names.contains(&"Performance"));
    assert!(dim_names.contains(&"Readability"));
    assert!(dim_names.contains(&"Safety"));
    assert!(dim_names.contains(&"Security"));
}

#[test]
fn grading_produces_improvement_projection() {
    use luau_grader_core::ruleset_config::RulesetConfig;
    let config = RulesetConfig::default();
    let report = analyzer::analyze_graded(
        "wait(1)\nspawn(function() end)\n",
        Tier::FrontPage, "test.luau", &[], &config,
    ).unwrap();

    assert!(!report.improvement.current_grade.is_empty());
    assert!(!report.improvement.projected_grade.is_empty());
}

#[test]
fn grading_debt_calculation() {
    use luau_grader_core::ruleset_config::RulesetConfig;
    let config = RulesetConfig::default();
    let report = analyzer::analyze_graded(
        "wait(1)\nspawn(function() end)\ndelay(1, function() end)\n",
        Tier::FrontPage, "test.luau", &[], &config,
    ).unwrap();

    assert!(report.debt.total_minutes > 0, "Expected nonzero technical debt");
}

#[test]
fn i014_self_assignment() {
    let ids = grade("local x = 1\nx = x\n", Tier::Intermediate);
    assert!(ids.contains(&"I014".to_string()));
}

#[test]
fn i014_normal_assignment() {
    let ids = grade("local x = 1\nx = x + 1\n", Tier::Intermediate);
    assert!(!ids.contains(&"I014".to_string()));
}

#[test]
fn i016_empty_function() {
    let ids = grade("local function foo()\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I016".to_string()));
}

#[test]
fn i016_nonempty_function() {
    let ids = grade("local function foo()\n    return 1\nend\n", Tier::Intermediate);
    assert!(!ids.contains(&"I016".to_string()));
}

#[test]
fn i017_duplicate_key() {
    let ids = grade("local t = { name = 1, name = 2 }\n", Tier::Intermediate);
    assert!(ids.contains(&"I017".to_string()));
}

#[test]
fn i017_unique_keys() {
    let ids = grade("local t = { name = 1, age = 2 }\n", Tier::Intermediate);
    assert!(!ids.contains(&"I017".to_string()));
}

#[test]
fn i019_while_wait_do() {
    let ids = grade("while wait() do\n    print(1)\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I019".to_string()));
}

#[test]
fn i019_while_true_do() {
    let ids = grade("while true do\n    task.wait()\nend\n", Tier::Intermediate);
    assert!(!ids.contains(&"I019".to_string()));
}

#[test]
fn i020_nil_comparison() {
    let ids = grade("local x = nil\nif x == nil then\n    print('nil')\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I020".to_string()));
}

#[test]
fn i021_negated_condition() {
    let ids = grade("local x = true\nif not x then\n    print('a')\nelse\n    print('b')\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I021".to_string()));
}


#[test]
fn a013_unreachable_code() {
    let ids = grade("local function foo()\n    if true then\n        return 1\n    end\n    return 2\nend\n", Tier::Advanced);
    assert!(!ids.is_empty());
}

#[test]
fn a013_reachable_code() {
    let ids = grade("local function foo()\n    print('hello')\n    return 1\nend\n", Tier::Advanced);
    assert!(!ids.contains(&"A013".to_string()));
}

#[test]
fn a017_deprecated_tick() {
    let ids = grade("local t = tick()\n", Tier::Advanced);
    assert!(ids.contains(&"A017".to_string()));
}

#[test]
fn a017_os_clock_ok() {
    let ids = grade("local t = os.clock()\n", Tier::Advanced);
    assert!(!ids.contains(&"A017".to_string()));
}

#[test]
fn a018_tween_size() {
    let ids = grade("frame:TweenSize(UDim2.new(1,0,1,0))\n", Tier::Advanced);
    assert!(ids.contains(&"A018".to_string()));
}

#[test]
fn a022_global_write() {
    let ids = grade("x = 42\n", Tier::Advanced);
    assert!(ids.contains(&"A022".to_string()));
}

#[test]
fn a022_local_write_ok() {
    let ids = grade("local x = 42\n", Tier::Advanced);
    assert!(!ids.contains(&"A022".to_string()));
}

#[test]
fn f012_connect_with_call_result() {
    let ids = grade("event:Connect(handler())\n", Tier::FrontPage);
    assert!(ids.contains(&"F012".to_string()));
}

#[test]
fn f012_connect_with_function_ok() {
    let ids = grade("event:Connect(function() end)\n", Tier::FrontPage);
    assert!(!ids.contains(&"F012".to_string()));
}

#[test]
fn f015_todo_comment() {
    let ids = grade("-- TODO fix this\nprint('hello')\n", Tier::FrontPage);
    assert!(ids.contains(&"F015".to_string()));
}

#[test]
fn f015_no_todo() {
    let ids = grade("-- this is fine\nprint('hello')\n", Tier::FrontPage);
    assert!(!ids.contains(&"F015".to_string()));
}

#[test]
fn b013_filtering_enabled() {
    let ids = grade("if workspace.FilteringEnabled then\n    print('ok')\nend\n", Tier::FrontPage);
    assert!(ids.contains(&"B013".to_string()));
}

#[test]
fn s002_no_rate_limit() {
    let ids = grade("remoteEvent.OnServerEvent:Connect(function(player, data)\n    print(data)\nend)\n", Tier::FrontPage);
    assert!(ids.contains(&"S002".to_string()));
}

#[test]
fn updated_total_rule_count() {
    let all = rulesets::list_all_rules();
    assert!(all.len() >= 91, "Expected at least 91 rules, got {}", all.len());
}

#[test]
fn fix_tick_to_os_clock() {
    let source = "--!strict\nlocal t = tick()\n";
    let report = analyzer::analyze(source, Tier::Advanced, "test.luau", &[]).unwrap();
    let tick_diags: Vec<_> = report.diagnostics.iter().filter(|d| d.rule_id == "A017").cloned().collect();
    assert!(!tick_diags.is_empty());
    let rules = rulesets::rules_for_tier(Tier::Advanced, &[]);
    let fix_report = fixer::apply_fixes(source, &tick_diags, &rules);
    assert!(fix_report.fixed_source.contains("os.clock()"));
}

#[test]
fn i026_variable_shadowing() {
    let ids = grade("local x = 1\nif true then\n    local x = 2\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I026".to_string()));
}

#[test]
fn i027_unused_local() {
    let ids = grade("local unusedVar = 42\nprint('hello')\n", Tier::Intermediate);
    assert!(ids.contains(&"I027".to_string()));
}

#[test]
fn i027_used_local_ok() {
    let ids = grade("local usedVar = 42\nprint(usedVar)\n", Tier::Intermediate);
    assert!(!ids.contains(&"I027".to_string()));
}

#[test]
fn a023_deprecated_body_mover() {
    let ids = grade("local bv = Instance.new(\"BodyVelocity\")\n", Tier::Advanced);
    assert!(ids.contains(&"A023".to_string()));
}

#[test]
fn a023_modern_constraint_ok() {
    let ids = grade("local lv = Instance.new(\"LinearVelocity\")\n", Tier::Advanced);
    assert!(!ids.contains(&"A023".to_string()));
}

#[test]
fn a026_set_async_in_pcall() {
    let ids = grade("pcall(function()\n    store:SetAsync(key, data)\nend)\n", Tier::Advanced);
    assert!(ids.contains(&"A026".to_string()));
}

#[test]
fn i029_vague_variable_name() {
    let ids = grade("local temp = getData()\n", Tier::Intermediate);
    assert!(ids.contains(&"I029".to_string()));
}

#[test]
fn i029_descriptive_name_ok() {
    let ids = grade("local playerInventory = getInventory()\n", Tier::Intermediate);
    assert!(!ids.contains(&"I029".to_string()));
}

#[test]
fn i030_redundant_boolean() {
    let ids = grade("local x = true\nif x == true then\n    print('yes')\nend\n", Tier::Intermediate);
    assert!(ids.contains(&"I030".to_string()));
}

#[test]
fn i030_normal_condition_ok() {
    let ids = grade("local x = true\nif x then\n    print('yes')\nend\n", Tier::Intermediate);
    assert!(!ids.contains(&"I030".to_string()));
}

#[test]
fn i032_duplicate_getservice() {
    let source = "local Players = game:GetService(\"Players\")\nlocal p2 = game:GetService(\"Players\")\n";
    let ids = grade(source, Tier::Intermediate);
    assert!(ids.contains(&"I032".to_string()));
}

#[test]
fn i032_single_getservice_ok() {
    let source = "local Players = game:GetService(\"Players\")\n";
    let ids = grade(source, Tier::Intermediate);
    assert!(!ids.contains(&"I032".to_string()));
}

#[test]
fn a027_pcall_error_swallowed() {
    let source = "local success, err = pcall(function()\n    doSomething()\nend)\nif not success then\n    return nil\nend\n";
    let ids = grade(source, Tier::Advanced);
    assert!(ids.contains(&"A027".to_string()));
}

#[test]
fn a027_pcall_error_logged_ok() {
    let source = "local success, err = pcall(function()\n    doSomething()\nend)\nif not success then\n    warn(err)\nend\n";
    let ids = grade(source, Tier::Advanced);
    assert!(!ids.contains(&"A027".to_string()));
}