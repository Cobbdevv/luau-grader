use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use colored::Colorize;
use luau_grader_core::config::Tier;
use luau_grader_core::report::Severity;
use luau_grader_core::ruleset_config::RulesetConfig;
use luau_grader_core::{analyzer, batch, fixer, rulesets};

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
    ListRules,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { path, tier, format, config } => cmd_check(&path, &tier, &format, config),
        Commands::CheckDir { path, tier, format, recursive, config } => cmd_check_dir(&path, &tier, &format, recursive, config),
        Commands::Fix { path, tier, dry_run, config } => cmd_fix(&path, &tier, dry_run, config),
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
        println!("  {} {} — {loc}: {}", "fix".green(), fix.rule_id.cyan(), fix.description);
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

fn cmd_list_rules() -> ExitCode {
    let rules = rulesets::list_all_rules();
    println!("{}", "Available Rules".bold().underline());
    let mut current_tier = String::new();
    for rule in &rules {
        if rule.tier != current_tier {
            current_tier = rule.tier.clone();
            println!("\n  {}", current_tier.bold());
        }
        let fix_marker = if rule.fixable { " ✓fix".green().to_string() } else { String::new() };
        println!("    {} {} — {}{}", rule.id.cyan(), format!("[{}]", rule.category).dimmed(), rule.description, fix_marker);
    }
    ExitCode::SUCCESS
}

fn print_report(report: &luau_grader_core::report::Report) {
    println!("\n  {} — {}", report.file.bold(), report.tier);
    for diag in &report.diagnostics {
        let sev = match diag.severity {
            Severity::Error => "✗".red(),
            Severity::Warning => "⚠".yellow(),
            Severity::Info => "ℹ".blue(),
        };
        let loc = diag.span.as_ref().map(|s| format!("Line {}:{}", s.line, s.column)).unwrap_or_default();
        println!("  {} {}  {:<18} {} {}", sev, diag.rule_id.cyan(), diag.category.dimmed(), loc.dimmed(), diag.message);
        if let Some(fix) = &diag.suggestion {
            println!("    {} {}", "Fix:".dimmed(), fix.green());
        }
    }

    let errors = report.diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = report.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
    let score = 100u32.saturating_sub((errors as u32) * 15 + (warnings as u32) * 5);
    let grade_str = match score {
        97..=100 => "A+", 93..=96 => "A", 90..=92 => "A-",
        87..=89 => "B+", 83..=86 => "B", 80..=82 => "B-",
        77..=79 => "C+", 73..=76 => "C", 70..=72 => "C-",
        67..=69 => "D+", 63..=66 => "D", 60..=62 => "D-",
        _ => "F",
    };
    println!("  {} {} ({}/100) — {} errors, {} warnings",
        "Grade:".bold(), grade_str.bold(), score,
        errors.to_string().red(), warnings.to_string().yellow());
}
