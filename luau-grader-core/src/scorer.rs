use crate::grade::*;
use crate::metrics::{FileMetrics, ScriptType};
use crate::report::Diagnostic;

pub fn calculate_grade(
    diagnostics: &[Diagnostic],
    metrics: &FileMetrics,
    file: &str,
    tier: &str,
) -> GradeReport {
    let weights = get_weights(metrics.script_type);
    let structure = score_structure(diagnostics, metrics);
    let api = score_api_correctness(diagnostics);
    let error_handling = score_error_handling(diagnostics, metrics);
    let performance = score_performance(diagnostics);
    let readability = score_readability(diagnostics, metrics);
    let safety = score_safety(diagnostics, metrics);
    let security = score_security(diagnostics, metrics);

    let dimensions = vec![
        structure, api, error_handling, performance, readability, safety, security,
    ];

    let overall_score: f64 = dimensions
        .iter()
        .zip(weights.iter())
        .map(|(dim, weight)| dim.score as f64 * weight)
        .sum();

    let overall_score = overall_score.clamp(0.0, 100.0);
    let grade = score_to_grade(overall_score);
    let strengths = detect_strengths(diagnostics, metrics);
    let key_issues = detect_key_issues(&dimensions, diagnostics);
    let function_grades: Vec<FunctionGrade> = metrics.functions.iter().map(grade_function).collect();
    let debt = calculate_debt(diagnostics, metrics);
    let improvement = project_improvement(overall_score, &dimensions, &debt);

    GradeReport {
        file: file.to_string(),
        tier: tier.to_string(),
        grade,
        overall_score,
        script_type: metrics.script_type,
        dimensions,
        function_grades,
        strengths,
        key_issues,
        improvement,
        debt,
        detected_patterns: metrics.detected_patterns.clone(),
        diagnostics: diagnostics.to_vec(),
        metrics: metrics.clone(),
    }
}

fn get_weights(script_type: ScriptType) -> [f64; 7] {
    match script_type {
        ScriptType::ServerScript => [0.20, 0.15, 0.15, 0.15, 0.20, 0.10, 0.05],
        ScriptType::ClientScript => [0.20, 0.15, 0.10, 0.15, 0.20, 0.15, 0.05],
        ScriptType::ModuleScript | ScriptType::SharedModule => [0.25, 0.15, 0.10, 0.15, 0.25, 0.10, 0.00],
        ScriptType::Plugin => [0.20, 0.15, 0.15, 0.15, 0.20, 0.10, 0.05],
        ScriptType::Unknown => [0.20, 0.15, 0.15, 0.15, 0.20, 0.10, 0.05],
    }
}

fn score_structure(diagnostics: &[Diagnostic], metrics: &FileMetrics) -> DimensionScore {
    let mut score: i32 = 100;
    let mut deductions = Vec::new();
    let mut bonuses = Vec::new();

    if metrics.total_lines > 1000 {
        score -= 25;
        deductions.push(format!("File is {} lines (very large)", metrics.total_lines));
    } else if metrics.total_lines > 500 {
        score -= 15;
        deductions.push(format!("File is {} lines", metrics.total_lines));
    }

    if metrics.function_count == 0 && metrics.total_lines > 20 {
        score -= 25;
        deductions.push("No functions defined, all code in global scope".to_string());
    }

    if metrics.function_count == 1 && metrics.total_lines > 100 {
        score -= 15;
        deductions.push("Single function contains all logic".to_string());
    }

    for func in &metrics.functions {
        if func.line_count > 100 {
            score -= 15;
            deductions.push(format!("{}() is {} lines", func.name, func.line_count));
        } else if func.line_count > 50 {
            score -= 8;
        }

        if func.cyclomatic_complexity > 20 {
            score -= 15;
            deductions.push(format!("{}() has complexity {}", func.name, func.cyclomatic_complexity));
        } else if func.cyclomatic_complexity > 10 {
            score -= 8;
        }

        if func.param_count > 5 {
            score -= 5;
            deductions.push(format!("{}() has {} parameters", func.name, func.param_count));
        }
    }

    let avg_nesting: f64 = if metrics.functions.is_empty() {
        0.0
    } else {
        metrics.functions.iter().map(|f| f.max_nesting as f64).sum::<f64>() / metrics.functions.len() as f64
    };

    if avg_nesting > 3.0 {
        score -= 10;
        deductions.push(format!("Average nesting depth {:.1}", avg_nesting));
    }

    let max_nesting = metrics.functions.iter().map(|f| f.max_nesting).max().unwrap_or(0);
    if max_nesting > 6 {
        score -= 10;
        deductions.push(format!("Maximum nesting depth {}", max_nesting));
    }

    if !metrics.duplicate_function_pairs.is_empty() {
        let penalty = (metrics.duplicate_function_pairs.len() as i32 * 15).min(30);
        score -= penalty;
        for (a, b) in &metrics.duplicate_function_pairs {
            deductions.push(format!("{}() and {}() have highly similar code (duplication)", a, b));
        }
    }

    if metrics.short_function_name_count > 0 {
        let penalty = (metrics.short_function_name_count as i32 * 5).min(20);
        score -= penalty;
        deductions.push(format!("{} functions have cryptic names (1-2 chars)", metrics.short_function_name_count));
    }

    let code_quality_issues = diagnostics.iter()
        .filter(|d| d.rule_id == "I026" || d.rule_id == "I027" || d.rule_id == "I028")
        .count();

    if code_quality_issues > 0 {
        let penalty = (code_quality_issues as i32 * 2).min(20);
        score -= penalty;
        deductions.push(format!("{} code quality issues (shadowed vars, unused locals, repeated chains)", code_quality_issues));
    }

    let data_issues = diagnostics.iter()
        .filter(|d| d.category == "Data Persistence")
        .count();

    if data_issues > 0 {
        let penalty = (data_issues as i32 * 5).min(20);
        score -= penalty;
        deductions.push(format!("{} data persistence issues", data_issues));
    }

    let dup_service_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I032")
        .count();

    if dup_service_count > 0 {
        let penalty = (dup_service_count as i32 * 3).min(15);
        score -= penalty;
        deductions.push(format!("{} duplicate GetService calls (declare services once at the top)", dup_service_count));
    }

    let funcs_with_guards = metrics.functions.iter().filter(|f| f.guard_clause_count > 0).count();

    if funcs_with_guards >= 2 && metrics.function_count > 0 {
        let guard_bonus = (funcs_with_guards as i32 * 2).min(8);
        score += guard_bonus;
        bonuses.push(format!("{} functions use guard clauses for flat control flow", funcs_with_guards));
    }

    if metrics.code_organization_score >= 0.8 {
        score += 5;
        bonuses.push("Well-organized code structure (services, requires, functions, init)".to_string());
    } else if metrics.code_organization_score < 0.5 && metrics.function_count > 0 && metrics.service_count > 0 {
        score -= 8;
        deductions.push("Code structure is disorganized (services, functions, and connections mixed together)".to_string());
    }

    if metrics.functions.iter().all(|f| f.line_count < 30) && metrics.function_count > 0 {
        score += 3;
        bonuses.push("All functions under 30 lines".to_string());
    }

    if metrics.functions.iter().all(|f| f.cyclomatic_complexity < 5) && metrics.function_count > 0 {
        score += 3;
        bonuses.push("Low complexity across all functions".to_string());
    }

    if metrics.functions.iter().all(|f| f.param_count <= 3) && metrics.function_count > 0 {
        score += 3;
        bonuses.push("Clean parameter signatures".to_string());
    }

    DimensionScore {
        name: "Structure".to_string(),
        score: score.clamp(0, 100) as u8,
        weight: 0.0,
        deductions,
        bonuses,
    }
}

fn score_api_correctness(diagnostics: &[Diagnostic]) -> DimensionScore {
    let mut score: i32 = 100;
    let mut deductions = Vec::new();
    let bonuses = Vec::new();

    let mut deprecated_count = 0;
    let mut arg_count_errors = 0;

    for diag in diagnostics {
        if diag.category == "API Deprecation" {
            deprecated_count += 1;
            match diag.severity {
                crate::report::Severity::Error => score -= 10,
                crate::report::Severity::Warning => score -= 5,
                crate::report::Severity::Info => score -= 2,
            }
        }
        if diag.category == "Common Bugs" && diag.message.contains("expects") && diag.message.contains("argument") {
            arg_count_errors += 1;
            score -= 10;
        }
    }

    if deprecated_count > 0 {
        deductions.push(format!("{} deprecated API calls", deprecated_count));
    }
    if arg_count_errors > 0 {
        deductions.push(format!("{} incorrect argument counts", arg_count_errors));
    }

    DimensionScore {
        name: "API Correctness".to_string(),
        score: score.clamp(0, 100) as u8,
        weight: 0.0,
        deductions,
        bonuses,
    }
}

fn score_error_handling(diagnostics: &[Diagnostic], metrics: &FileMetrics) -> DimensionScore {
    let mut score: i32 = 100;
    let mut deductions = Vec::new();
    let mut bonuses = Vec::new();

    let needs_error_handling = metrics.has_strict_mode
        || metrics.services_used.contains(&"DataStoreService".to_string())
        || metrics.services_used.contains(&"HttpService".to_string())
        || metrics.services_used.contains(&"MarketplaceService".to_string());

    let is_server = metrics.script_type == ScriptType::ServerScript;

    let has_any_pcall = metrics.functions.iter().any(|f| f.has_error_handling);

    if needs_error_handling && !has_any_pcall {
        score -= 35;
        deductions.push("Uses services that can fail but has no pcall/xpcall".to_string());
    }

    let mut bare_pcall_count = 0;
    let mut ffc_chain_count = 0;
    let mut swallowed_error_count = 0;

    for diag in diagnostics {
        if diag.category == "Error Handling" && diag.rule_id != "A027" {
            bare_pcall_count += 1;
            score -= 10;
        }
        if diag.rule_id == "F005" {
            ffc_chain_count += 1;
            score -= 10;
        }
        if diag.rule_id == "A027" {
            swallowed_error_count += 1;
        }
    }

    if bare_pcall_count > 0 {
        deductions.push(format!("{} pcall results not checked", bare_pcall_count));
    }
    if ffc_chain_count > 0 {
        deductions.push(format!("{} FindFirstChild chains without nil check", ffc_chain_count));
    }
    if swallowed_error_count > 0 {
        let penalty = (swallowed_error_count * 5).min(20);
        score -= penalty;
        deductions.push(format!("{} pcall errors captured but silently swallowed (never logged)", swallowed_error_count));
    }

    if !metrics.functions.is_empty() && !metrics.functions.iter().any(|f| f.has_error_handling) {
        let penalty = if is_server { 25 } else { 15 };
        score -= penalty;
        deductions.push("No error handling in any function".to_string());
    }

    let funcs_with_handling = metrics.functions.iter().filter(|f| f.has_error_handling).count();
    if funcs_with_handling > 0 && metrics.function_count > 0 {
        let ratio = funcs_with_handling as f64 / metrics.function_count as f64;
        if ratio > 0.5 {
            score += 5;
            bonuses.push("Good error handling coverage".to_string());
        }
    }

    if ffc_chain_count == 0 {
        score += 3;
        bonuses.push("All FindFirstChild results properly checked".to_string());
    }

    DimensionScore {
        name: "Error Handling".to_string(),
        score: score.clamp(0, 100) as u8,
        weight: 0.0,
        deductions,
        bonuses,
    }
}

fn score_performance(diagnostics: &[Diagnostic]) -> DimensionScore {
    let mut score: i32 = 100;
    let mut deductions = Vec::new();
    let bonuses = Vec::new();

    let mut perf_issues = 0;
    for diag in diagnostics {
        if diag.category == "Performance" {
            perf_issues += 1;
            match diag.severity {
                crate::report::Severity::Error => score -= 15,
                crate::report::Severity::Warning => score -= 8,
                crate::report::Severity::Info => score -= 3,
            }
        }
    }

    if perf_issues > 0 {
        deductions.push(format!("{} performance issues", perf_issues));
    }

    DimensionScore {
        name: "Performance".to_string(),
        score: score.clamp(0, 100) as u8,
        weight: 0.0,
        deductions,
        bonuses,
    }
}

fn score_readability(diagnostics: &[Diagnostic], metrics: &FileMetrics) -> DimensionScore {
    let mut score: i32 = 100;
    let mut deductions = Vec::new();
    let mut bonuses = Vec::new();

    let single_letter_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I002")
        .count();

    if single_letter_count > 0 {
        let penalty = (single_letter_count as i32 * 3).min(40);
        score -= penalty;
        deductions.push(format!("{} single-letter variable names", single_letter_count));
    }

    let magic_number_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I025")
        .count();

    if magic_number_count > 0 {
        let penalty = (magic_number_count as i32 * 2).min(30);
        score -= penalty;
        deductions.push(format!("{} magic numbers without named constants", magic_number_count));
    }

    let print_warn_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I009")
        .count();

    if print_warn_count > 0 {
        let penalty = (print_warn_count as i32 * 2).min(10);
        score -= penalty;
        deductions.push(format!("{} debug print/warn calls left in code", print_warn_count));
    }

    let unused_var_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I027")
        .count();

    if unused_var_count > 0 {
        let penalty = (unused_var_count as i32 * 3).min(20);
        score -= penalty;
        deductions.push(format!("{} unused local variables", unused_var_count));
    }

    let shadowing_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I026")
        .count();

    if shadowing_count > 0 {
        let penalty = (shadowing_count as i32 * 3).min(15);
        score -= penalty;
        deductions.push(format!("{} shadowed variable names", shadowing_count));
    }

    let repeated_chain_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I028")
        .count();

    if repeated_chain_count > 0 {
        let penalty = (repeated_chain_count as i32 * 3).min(15);
        score -= penalty;
        deductions.push(format!("{} repeated deep property chains (should be cached)", repeated_chain_count));
    }

    let vague_name_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I029")
        .count();

    if vague_name_count > 0 {
        let penalty = (vague_name_count as i32 * 2).min(20);
        score -= penalty;
        deductions.push(format!("{} vague variable names (data, temp, result, etc.)", vague_name_count));
    }

    let bool_compare_count = diagnostics.iter()
        .filter(|d| d.rule_id == "I030")
        .count();

    if bool_compare_count > 0 {
        let penalty = (bool_compare_count as i32 * 2).min(10);
        score -= penalty;
        deductions.push(format!("{} redundant boolean comparisons (== true / == false)", bool_compare_count));
    }

    if metrics.avg_function_length > 40.0 {
        score -= 10;
        deductions.push(format!("Average function length {:.0} lines", metrics.avg_function_length));
    }

    for func in &metrics.functions {
        if func.cognitive_complexity > 15 {
            score -= 5;
            deductions.push(format!("{}() has cognitive complexity {}", func.name, func.cognitive_complexity));
        }
    }

    if metrics.naming_quality < 2.0 {
        score -= 30;
        deductions.push("Variable names are extremely short on average".to_string());
    } else if metrics.naming_quality < 2.5 {
        score -= 20;
        deductions.push("Variable names are very short on average".to_string());
    } else if metrics.naming_quality < 3.0 {
        score -= 10;
        deductions.push("Variable names are too short on average".to_string());
    }

    if metrics.short_function_name_count > 0 {
        let penalty = (metrics.short_function_name_count as i32 * 3).min(20);
        score -= penalty;
        deductions.push(format!("{} functions have cryptic names (1-2 chars)", metrics.short_function_name_count));
    }

    if metrics.total_lines > 50 && metrics.comment_line_count == 0 {
        score -= 15;
        deductions.push("No comments in a file with 50+ lines".to_string());
    }

    if metrics.consistency_score < 0.8 && metrics.consistency_score > 0.0 {
        score -= 10;
        deductions.push("Inconsistent API style (mix of old and modern patterns)".to_string());
    }

    if metrics.naming_style_consistency < 0.6 {
        score -= 10;
        deductions.push("Inconsistent naming conventions (mix of camelCase and snake_case)".to_string());
    } else if metrics.naming_style_consistency >= 0.9 && metrics.naming_quality >= 4.0 {
        score += 3;
        bonuses.push("Consistent naming conventions".to_string());
    }

    let total_guards: usize = metrics.functions.iter().map(|f| f.guard_clause_count).sum();
    if total_guards >= 3 {
        score += 5;
        bonuses.push("Uses guard clauses for clean, flat control flow".to_string());
    }

    if metrics.naming_quality >= 6.0 {
        score += 3;
        bonuses.push("Descriptive variable names".to_string());
    }

    if metrics.has_strict_mode {
        score += 3;
        bonuses.push("Uses strict mode".to_string());
    }

    let comment_ratio = if metrics.total_lines > 0 {
        metrics.comment_line_count as f64 / metrics.total_lines as f64
    } else {
        0.0
    };

    if comment_ratio >= 0.05 && comment_ratio <= 0.20 {
        score += 3;
        bonuses.push("Good comment-to-code ratio".to_string());
    }

    DimensionScore {
        name: "Readability".to_string(),
        score: score.clamp(0, 100) as u8,
        weight: 0.0,
        deductions,
        bonuses,
    }
}

fn score_safety(diagnostics: &[Diagnostic], metrics: &FileMetrics) -> DimensionScore {
    let mut score: i32 = 100;
    let mut deductions = Vec::new();
    let mut bonuses = Vec::new();

    let is_server = metrics.script_type == ScriptType::ServerScript;

    let mut memory_issues = 0;
    let mut has_unstored_connection = false;
    let mut has_parent_nil = false;
    let mut has_unstored_clone = false;

    for diag in diagnostics {
        if diag.category == "Memory Management" {
            memory_issues += 1;
            score -= 10;
            if diag.rule_id == "A002" { has_unstored_connection = true; }
            if diag.rule_id == "A009" { has_unstored_clone = true; }
        }
        if diag.rule_id == "F002" {
            has_parent_nil = true;
            score -= 10;
        }
    }

    if memory_issues > 0 {
        deductions.push(format!("{} memory management issues", memory_issues));
    }
    if has_unstored_connection {
        deductions.push("Connections created without cleanup".to_string());
    }
    if has_parent_nil {
        deductions.push("Parent set to nil without Destroy".to_string());
    }
    if has_unstored_clone {
        deductions.push("Clone result not stored".to_string());
    }

    if !metrics.has_strict_mode && is_server {
        score -= 15;
        deductions.push("Server script without --!strict mode".to_string());
    } else if !metrics.has_strict_mode && metrics.total_lines > 50 {
        score -= 8;
        deductions.push("No --!strict mode in a substantial file".to_string());
    }

    if !has_unstored_connection && metrics.detected_patterns.contains(&"Observer".to_string()) {
        score += 3;
        bonuses.push("All connections properly managed".to_string());
    }

    if metrics.detected_patterns.contains(&"Cleanup/Janitor".to_string()) {
        score += 3;
        bonuses.push("Uses cleanup pattern for resource management".to_string());
    }

    DimensionScore {
        name: "Safety".to_string(),
        score: score.clamp(0, 100) as u8,
        weight: 0.0,
        deductions,
        bonuses,
    }
}

fn score_security(diagnostics: &[Diagnostic], _metrics: &FileMetrics) -> DimensionScore {
    let mut score: i32 = 100;
    let mut deductions = Vec::new();
    let bonuses = Vec::new();

    for diag in diagnostics {
        if diag.rule_id.starts_with('S') {
            match diag.severity {
                crate::report::Severity::Error => {
                    score -= 20;
                    deductions.push(diag.message.clone());
                }
                crate::report::Severity::Warning => {
                    score -= 10;
                    deductions.push(diag.message.clone());
                }
                crate::report::Severity::Info => {
                    score -= 5;
                }
            }
        }
    }

    DimensionScore {
        name: "Security".to_string(),
        score: score.clamp(0, 100) as u8,
        weight: 0.0,
        deductions,
        bonuses,
    }
}

fn detect_strengths(_diagnostics: &[Diagnostic], metrics: &FileMetrics) -> Vec<String> {
    let mut strengths = Vec::new();

    if metrics.service_count > 0 && metrics.detected_patterns.contains(&"Module Pattern".to_string()) {
        strengths.push("Well-structured module with clear exports".to_string());
    }

    if metrics.has_strict_mode {
        strengths.push("Type-safe with strict mode enabled".to_string());
    }

    if metrics.consistency_score >= 0.95 && metrics.service_count > 0 {
        strengths.push("Consistent modern API usage throughout".to_string());
    }

    if metrics.functions.iter().all(|f| f.line_count < 30) && metrics.function_count >= 3 {
        strengths.push("Functions are well-decomposed and focused".to_string());
    }

    if metrics.functions.iter().all(|f| f.cyclomatic_complexity <= 5) && metrics.function_count > 0 {
        strengths.push("Low complexity across all functions, easy to test".to_string());
    }

    if metrics.functions.iter().any(|f| f.has_error_handling) {
        strengths.push("Error handling present for risky operations".to_string());
    }

    if metrics.naming_quality >= 6.0 {
        strengths.push("Descriptive naming conventions used throughout".to_string());
    }

    if metrics.detected_patterns.contains(&"Cleanup/Janitor".to_string()) {
        strengths.push("Proper resource cleanup implemented".to_string());
    }

    if metrics.duplicate_function_pairs.is_empty() && metrics.function_count >= 3 {
        strengths.push("No code duplication detected".to_string());
    }

    strengths
}

fn detect_key_issues(dimensions: &[DimensionScore], diagnostics: &[Diagnostic]) -> Vec<String> {
    let mut issues = Vec::new();

    for dim in dimensions {
        if dim.score < 60 {
            for deduction in &dim.deductions {
                issues.push(format!("[{}] {}", dim.name, deduction));
            }
        }
    }

    let error_count = diagnostics.iter()
        .filter(|d| d.severity == crate::report::Severity::Error)
        .count();

    if error_count > 0 {
        issues.insert(0, format!("{} critical errors that need immediate attention", error_count));
    }

    issues.truncate(5);
    issues
}

fn calculate_debt(diagnostics: &[Diagnostic], metrics: &FileMetrics) -> TechnicalDebt {
    let mut total_minutes: usize = 0;
    let mut breakdown = Vec::new();
    let mut category_minutes: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();

    for diag in diagnostics {
        let minutes = estimate_fix_time(diag);
        total_minutes += minutes;
        let entry = category_minutes.entry(diag.category.clone()).or_insert((0, 0));
        entry.0 += minutes;
        entry.1 += 1;
    }

    for func in &metrics.functions {
        if func.cyclomatic_complexity > 15 {
            total_minutes += 20;
            let entry = category_minutes.entry("Refactoring".to_string()).or_insert((0, 0));
            entry.0 += 20;
            entry.1 += 1;
        }
    }

    if !metrics.duplicate_function_pairs.is_empty() {
        let dup_minutes = metrics.duplicate_function_pairs.len() * 15;
        total_minutes += dup_minutes;
        let entry = category_minutes.entry("Code Duplication".to_string()).or_insert((0, 0));
        entry.0 += dup_minutes;
        entry.1 += metrics.duplicate_function_pairs.len();
    }

    for (category, (minutes, count)) in category_minutes {
        breakdown.push(DebtItem { category, minutes, count });
    }

    breakdown.sort_by(|a, b| b.minutes.cmp(&a.minutes));

    TechnicalDebt {
        total_minutes,
        breakdown,
        top_investments: Vec::new(),
    }
}

fn estimate_fix_time(diag: &crate::report::Diagnostic) -> usize {
    let mut base = match diag.category.as_str() {
        "API Deprecation" => 2,
        "Common Bugs" => {
            if diag.rule_id.starts_with("B01") { 2 } else { 4 }
        }
        "Performance" => 5,
        "Memory Management" => 5,
        "Error Handling" => 8,
        "Code Style" => 1,
        "Code Quality" => 2,
        "Data Persistence" => 15,
        "Module Architecture" => 10,
        "Security" => 15,
        _ => 3,
    };
    
    if diag.severity == crate::report::Severity::Info {
        base = std::cmp::min(base, 2);
    }
    
    base
}

fn project_improvement(
    current_score: f64,
    dimensions: &[DimensionScore],
    _debt: &TechnicalDebt,
) -> ImprovementProjection {
    let current_grade = score_to_grade(current_score);
    let mut fixes = Vec::new();

    let weights = [0.20, 0.15, 0.15, 0.15, 0.20, 0.10, 0.05];

    for (dim_idx, dim) in dimensions.iter().enumerate() {
        if dim.score < 100 {
            let weight = weights.get(dim_idx).copied().unwrap_or(0.10);
            for deduction in &dim.deductions {
                let gap = 100 - dim.score as i32;
                let estimated_fix_value = match gap {
                    0..=10 => 3.0,
                    11..=30 => 5.0,
                    _ => 8.0,
                };
                let weighted_impact = estimated_fix_value * weight * 2.0;

                let effort = if deduction.contains("complexity")
                    || deduction.contains("duplication")
                    || deduction.contains("lines")
                {
                    20
                } else if deduction.contains("naming")
                    || deduction.contains("variable")
                    || deduction.contains("boolean")
                    || deduction.contains("GetService")
                {
                    5
                } else {
                    10
                };

                fixes.push(ProjectedFix {
                    description: deduction.clone(),
                    score_impact: weighted_impact,
                    dimension: dim.name.clone(),
                    effort_minutes: effort,
                });
            }
        }
    }

    fixes.sort_by(|a, b| {
        let a_ratio = a.score_impact / a.effort_minutes as f64;
        let b_ratio = b.score_impact / b.effort_minutes as f64;
        b_ratio.partial_cmp(&a_ratio).unwrap_or(std::cmp::Ordering::Equal)
    });
    fixes.truncate(3);

    let projected_score = (current_score + fixes.iter().map(|f| f.score_impact).sum::<f64>()).min(100.0);
    let projected_grade = score_to_grade(projected_score);
    let effort_minutes = fixes.iter().map(|f| f.effort_minutes).sum();

    ImprovementProjection {
        current_grade,
        projected_grade,
        fixes_needed: fixes,
        effort_minutes,
    }
}
