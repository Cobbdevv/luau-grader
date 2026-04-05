use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;

use clap::{Parser, Subcommand};
use colored::Colorize;
use luau_grader_core::config::Tier;
use luau_grader_core::grade::GradeReport;
use luau_grader_core::report::Severity;
use luau_grader_core::ruleset_config::RulesetConfig;
use luau_grader_core::{analyzer, batch, fixer, rulesets};

const STACK_SIZE: usize = 8 * 1024 * 1024;

#[derive(Parser)]
#[command(name = "luau-grader", version, about = "Static analysis and grading for Luau code")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Check {
        path: PathBuf,
        #[arg(long, default_value = "front_page")]
        tier: String,
        #[arg(long, default_value = "text")]
        format: String,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    CheckDir {
        path: PathBuf,
        #[arg(long, default_value = "front_page")]
        tier: String,
        #[arg(long, default_value = "text")]
        format: String,
        #[arg(long)]
        recursive: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Fix {
        path: PathBuf,
        #[arg(long, default_value = "front_page")]
        tier: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Grade {
        path: PathBuf,
        #[arg(long, default_value = "front_page")]
        tier: String,
        #[arg(long, default_value = "text")]
        format: String,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Workspace {
        path: PathBuf,
    },
    ListRules,
}

fn main() -> ExitCode {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .expect("failed to spawn thread");
    child.join().expect("thread panicked")
}

fn run() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { path, tier, format, config } => cmd_check(&path, &tier, &format, config),
        Commands::CheckDir { path, tier, format, recursive, config } => cmd_check_dir(&path, &tier, &format, recursive, config),
        Commands::Fix { path, tier, dry_run, config } => cmd_fix(&path, &tier, dry_run, config),
        Commands::Grade { path, tier, format, config } => cmd_grade(&path, &tier, &format, config),
        Commands::Workspace { path } => cmd_workspace(&path),
        Commands::ListRules => cmd_list_rules(),
    }
}

fn load_config(explicit: Option<PathBuf>, fallback_path: &Path) -> RulesetConfig {
    if let Some(path) = explicit {
        match RulesetConfig::load(&path) {
            Ok(c) => return c,
            Err(e) => { eprintln!("{} {e}", "config error:".red()); }
        }
    }
    RulesetConfig::find_in_ancestors(fallback_path).unwrap_or_default()
}

fn parse_tier(s: &str, config: &RulesetConfig) -> Result<Tier, String> {
    let tier_str = config.tier.as_deref().unwrap_or(s);
    tier_str.parse()
}

fn cmd_check(path: &Path, tier_str: &str, format: &str, config_path: Option<PathBuf>) -> ExitCode {
    let config = load_config(config_path, path);
    let tier = match parse_tier(tier_str, &config) {
        Ok(t) => t,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let report = match analyzer::analyze_with_config(&source, tier, &file_name, &config.disabled_rules, &config) {
        Ok(r) => r,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    if format == "json" {
        println!("{}", report.to_json().unwrap_or_default());
    } else {
        print_report(&report);
    }

    if report.diagnostics.iter().any(|d| d.severity == Severity::Error) { ExitCode::from(2) }
    else if !report.diagnostics.is_empty() { ExitCode::from(1) }
    else { ExitCode::SUCCESS }
}

fn cmd_check_dir(path: &Path, tier_str: &str, format: &str, recursive: bool, config_path: Option<PathBuf>) -> ExitCode {
    let config = load_config(config_path, path);
    let tier = match parse_tier(tier_str, &config) {
        Ok(t) => t,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let batch_report = match batch::analyze_directory(path, tier, &config.disabled_rules, recursive) {
        Ok(r) => r,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&batch_report).unwrap_or_default());
    } else {
        for report in &batch_report.reports {
            if !report.diagnostics.is_empty() { print_report(report); }
        }
        println!("\n{}", "Summary".bold().underline());
        println!("  Files: {}", batch_report.summary.total_files);
        println!("  Errors: {}", format!("{}", batch_report.summary.total_errors).red());
        println!("  Warnings: {}", format!("{}", batch_report.summary.total_warnings).yellow());
        println!("  Grade: {} ({})", batch_report.summary.grade.bold(), batch_report.summary.score);
    }

    if batch_report.summary.total_errors > 0 { ExitCode::from(2) }
    else if batch_report.summary.total_warnings > 0 { ExitCode::from(1) }
    else { ExitCode::SUCCESS }
}

fn cmd_fix(path: &Path, tier_str: &str, dry_run: bool, config_path: Option<PathBuf>) -> ExitCode {
    let config = load_config(config_path, path);
    let tier = match parse_tier(tier_str, &config) {
        Ok(t) => t,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let report = match analyzer::analyze_with_config(&source, tier, &file_name, &config.disabled_rules, &config) {
        Ok(r) => r,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let rules = rulesets::rules_for_tier_with_config(tier, &config.disabled_rules, &config);
    let fix_report = fixer::apply_fixes(&source, &report.diagnostics, &rules);

    if fix_report.applied.is_empty() {
        println!("{}", "No auto-fixes available.".dimmed());
        return ExitCode::SUCCESS;
    }

    for fix in &fix_report.applied {
        let loc = if fix.line > 0 { format!("line {}", fix.line) } else { "top".to_string() };
        println!("  {} {} - {loc}: {}", "fix".green(), fix.rule_id.cyan(), fix.description);
    }

    if dry_run {
        println!("\n{} {} fixes would be applied", "--dry-run:".yellow(), fix_report.applied.len());
    } else {
        if let Err(e) = std::fs::write(path, &fix_report.fixed_source) {
            eprintln!("{} {e}", "error writing file:".red());
            return ExitCode::from(2);
        }
        println!("\n{} {} fixes applied to {}", "done:".green().bold(), fix_report.applied.len(), file_name);
    }

    if !fix_report.unfixable.is_empty() {
        println!("\n{} {} issues require manual fixes", "note:".yellow(), fix_report.unfixable.len());
    }

    ExitCode::SUCCESS
}

fn cmd_grade(path: &Path, tier_str: &str, format: &str, config_path: Option<PathBuf>) -> ExitCode {
    let config = load_config(config_path, path);
    let tier = match parse_tier(tier_str, &config) {
        Ok(t) => t,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let grade_report = match analyzer::analyze_graded(&source, tier, &file_name, &config.disabled_rules, &config) {
        Ok(r) => r,
        Err(e) => { eprintln!("{} {e}", "error:".red()); return ExitCode::from(2); }
    };

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&grade_report).unwrap_or_default());
    } else {
        print_grade_report(&grade_report);
    }

    let has_errors = grade_report.diagnostics.iter().any(|d| d.severity == Severity::Error);
    if has_errors { ExitCode::from(2) }
    else if !grade_report.diagnostics.is_empty() { ExitCode::from(1) }
    else { ExitCode::SUCCESS }
}

fn cmd_list_rules() -> ExitCode {
    let rules = rulesets::list_all_rules();
    println!("{}", "Available Rules".bold().underline());
    let mut current_tier = String::new();
    for rule in &rules {
        if rule.tier != current_tier {
            current_tier = rule.tier.clone();
            println!("\n  {}", current_tier.bold());
        }
        let fix_marker = if rule.fixable { " [fixable]".green().to_string() } else { String::new() };
        println!("    {} {} - {}{}", rule.id.cyan(), format!("[{}]", rule.category).dimmed(), rule.description, fix_marker);
    }
    println!("\n  {} rules total", rules.len().to_string().bold());
    ExitCode::SUCCESS
}

fn cmd_workspace(path: &Path) -> ExitCode {
    if !path.is_dir() {
        println!("{}", "Error: Workspace path must be a directory".red());
        return ExitCode::FAILURE;
    }
    
    println!("{} workspace at {}", "Analyzing".cyan().bold(), path.display());
    
    match luau_grader_core::workspace::analyze_workspace(path) {
        Ok(graph) => {
            println!("Parsed {} luau files.", graph.nodes.len());
            
            let rules = luau_grader_core::workspace_rules::all_workspace_rules();
            let mut all_diags = Vec::new();
            
            for rule in rules {
                let diags = rule.analyze(&graph);
                all_diags.extend(diags);
            }
            
            if all_diags.is_empty() {
                println!("{}", "Workspace analysis clean. No cyclic dependencies or dead code found.".green());
                return ExitCode::SUCCESS;
            }
            
            let mut errors = 0;
            let mut warnings = 0;
            
            for diag in &all_diags {
                let sev_str = match diag.severity {
                    Severity::Error => { errors += 1; "Error".red() }
                    Severity::Warning => { warnings += 1; "Warning".yellow() }
                    Severity::Info => "Info".blue()
                };
                
                let file_str = if let Some(p) = &diag.file_path {
                    p.display().to_string()
                } else {
                    "Workspace".to_string()
                };
                
                println!("{}: [{}] {}", file_str.bold(), sev_str, diag.message);
            }
            
            println!("\nFound {} errors and {} warnings in workspace analysis.", errors, warnings);
            
            if errors > 0 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            println!("{} failed to analyze workspace: {}", "Error:".red(), e);
            ExitCode::FAILURE
        }
    }
}

fn print_report(report: &luau_grader_core::report::Report) {
    println!("\n  {} - {}", report.file.bold(), report.tier);
    for diag in &report.diagnostics {
        let sev = match diag.severity {
            Severity::Error => "E".red(),
            Severity::Warning => "W".yellow(),
            Severity::Info => "I".blue(),
        };
        let loc = diag.span.as_ref().map(|s| format!("L{}:{}", s.line, s.column)).unwrap_or_default();
        println!("  {} {}  {:<18} {} {}", sev, diag.rule_id.cyan(), diag.category.dimmed(), loc.dimmed(), diag.message);
        if let Some(fix) = &diag.suggestion {
            println!("    {} {}", "Fix:".dimmed(), fix.green());
        }
    }

    let errors = report.diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = report.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
    let score = 100u32.saturating_sub((errors as u32) * 15 + (warnings as u32) * 5);
    let grade_str = luau_grader_core::grade::score_to_grade(score as f64);
    println!("  {} {} ({}/100) - {} errors, {} warnings",
        "Grade:".bold(), grade_str.bold(), score,
        errors.to_string().red(), warnings.to_string().yellow());
}

fn print_grade_report(report: &GradeReport) {
    let script_type = format!("{:?}", report.script_type);
    println!("\n  {} - {}", report.file.bold(), script_type.dimmed());

    let grade_color = match report.grade.chars().next().unwrap_or('F') {
        'A' => report.grade.green().bold(),
        'B' => report.grade.blue().bold(),
        'C' => report.grade.yellow().bold(),
        'D' => report.grade.red().bold(),
        _ => report.grade.red().bold(),
    };
    println!("\n  {} {} ({:.0}/100)\n", "Grade:".bold(), grade_color, report.overall_score);

    println!("  {}", "Dimensions".bold().underline());
    for dim in &report.dimensions {
        let filled = (dim.score as usize) / 10;
        let empty = 10 - filled;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let bar_colored = if dim.score >= 80 { bar.green() }
            else if dim.score >= 60 { bar.yellow() }
            else { bar.red() };
        println!("    {:<20} {} {:>3}", dim.name, bar_colored, dim.score);
    }

    if !report.function_grades.is_empty() {
        println!("\n  {}", "Per-Function Grades".bold().underline());
        for fg in &report.function_grades {
            let grade_col = match fg.grade.chars().next().unwrap_or('F') {
                'A' => fg.grade.green(),
                'B' => fg.grade.blue(),
                'C' => fg.grade.yellow(),
                _ => fg.grade.red(),
            };
            println!("    {} {} {}  complexity:{}  lines:{}",
                fg.name.cyan(),
                format!("(line {})", fg.line).dimmed(),
                grade_col,
                fg.complexity, fg.lines);
        }
    }

    if report.debt.total_minutes > 0 {
        println!("\n  {} {} minutes", "Technical Debt:".bold().underline(), report.debt.total_minutes);
        for item in &report.debt.breakdown {
            if item.minutes > 0 {
                println!("    {:<24} {:>3} min  ({} issues)", item.category, item.minutes, item.count);
            }
        }
    }

    if !report.improvement.fixes_needed.is_empty() {
        println!("\n  {} {} -> {}",
            "Improvement Path".bold().underline(),
            report.improvement.current_grade.dimmed(),
            report.improvement.projected_grade.green().bold());
        for (i, fix) in report.improvement.fixes_needed.iter().enumerate() {
            println!("    {}. {}  {} ~{} min",
                i + 1,
                fix.description,
                format!("+{:.0} pts", fix.score_impact).green(),
                fix.effort_minutes);
        }
    }

    if !report.strengths.is_empty() {
        println!("\n  {}", "Strengths".bold().underline());
        for s in &report.strengths {
            println!("    {} {}", "+".green(), s);
        }
    }

    if !report.detected_patterns.is_empty() {
        println!("\n  {}", "Detected Patterns".bold().underline());
        for p in &report.detected_patterns {
            println!("    {} {}", "*".dimmed(), p);
        }
    }

    let errors = report.diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = report.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
    let infos = report.diagnostics.iter().filter(|d| d.severity == Severity::Info).count();

    println!("\n  {} {} diagnostics ({} errors, {} warnings, {} info)",
        "Summary:".bold(),
        report.diagnostics.len(),
        errors.to_string().red(),
        warnings.to_string().yellow(),
        infos.to_string().blue());
}
