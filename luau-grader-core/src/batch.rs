use std::path::Path;

use crate::analyzer;
use crate::config::Tier;
use crate::errors::GraderError;
use crate::report::{Report, Severity};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BatchSummary {
    pub total_files: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub grade: String,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchReport {
    pub reports: Vec<Report>,
    pub summary: BatchSummary,
}

pub fn analyze_directory(
    dir: &Path,
    tier: Tier,
    disabled_rules: &[String],
    recursive: bool,
) -> Result<BatchReport, GraderError> {
    let files = collect_files(dir, recursive)?;
    let mut reports = Vec::with_capacity(files.len());

    for path in &files {
        let source = std::fs::read_to_string(path)?;
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        reports.push(analyzer::analyze(&source, tier, &name, disabled_rules)?);
    }

    let total_errors = reports.iter()
        .flat_map(|r| &r.diagnostics)
        .filter(|d| d.severity == Severity::Error)
        .count();

    let total_warnings = reports.iter()
        .flat_map(|r| &r.diagnostics)
        .filter(|d| d.severity == Severity::Warning)
        .count();

    let score = 100u32
        .saturating_sub((total_errors as u32).saturating_mul(15))
        .saturating_sub((total_warnings as u32).saturating_mul(5));

    Ok(BatchReport {
        summary: BatchSummary {
            total_files: reports.len(),
            total_errors,
            total_warnings,
            grade: score_to_grade(score),
            score,
        },
        reports,
    })
}

fn collect_files(dir: &Path, recursive: bool) -> Result<Vec<std::path::PathBuf>, GraderError> {
    let mut files = Vec::new();
    walk_dir(dir, recursive, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_dir(dir: &Path, recursive: bool, out: &mut Vec<std::path::PathBuf>) -> Result<(), GraderError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && recursive {
            walk_dir(&path, recursive, out)?;
        } else if path.is_file()
            && let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "luau" || ext == "lua" {
                    out.push(path);
                }
        }
    }
    Ok(())
}

fn score_to_grade(score: u32) -> String {
    match score {
        97..=100 => "A+", 93..=96 => "A", 90..=92 => "A-",
        87..=89 => "B+", 83..=86 => "B", 80..=82 => "B-",
        77..=79 => "C+", 73..=76 => "C", 70..=72 => "C-",
        67..=69 => "D+", 63..=66 => "D", 60..=62 => "D-",
        _ => "F",
    }.to_string()
}
