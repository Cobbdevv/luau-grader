use serde::Serialize;
use crate::metrics::{FileMetrics, ScriptType};
use crate::report::Diagnostic;

#[derive(Debug, Clone, Serialize)]
pub struct DimensionScore {
    pub name: String,
    pub score: u8,
    pub weight: f64,
    pub deductions: Vec<String>,
    pub bonuses: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionGrade {
    pub name: String,
    pub line: usize,
    pub grade: String,
    pub score: f64,
    pub complexity: usize,
    pub cognitive: usize,
    pub lines: usize,
    pub issues: Vec<String>,
    pub strengths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebtItem {
    pub category: String,
    pub minutes: usize,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebtInvestment {
    pub description: String,
    pub minutes_to_fix: usize,
    pub grade_improvement: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechnicalDebt {
    pub total_minutes: usize,
    pub breakdown: Vec<DebtItem>,
    pub top_investments: Vec<DebtInvestment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectedFix {
    pub description: String,
    pub score_impact: f64,
    pub dimension: String,
    pub effort_minutes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImprovementProjection {
    pub current_grade: String,
    pub projected_grade: String,
    pub fixes_needed: Vec<ProjectedFix>,
    pub effort_minutes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GradeReport {
    pub file: String,
    pub tier: String,
    pub grade: String,
    pub overall_score: f64,
    pub script_type: ScriptType,
    pub dimensions: Vec<DimensionScore>,
    pub function_grades: Vec<FunctionGrade>,
    pub strengths: Vec<String>,
    pub key_issues: Vec<String>,
    pub improvement: ImprovementProjection,
    pub debt: TechnicalDebt,
    pub detected_patterns: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: FileMetrics,
}

pub fn score_to_grade(score: f64) -> String {
    match score as u8 {
        96..=100 => "A+".to_string(),
        92..=95 => "A".to_string(),
        88..=91 => "A-".to_string(),
        83..=87 => "B+".to_string(),
        77..=82 => "B".to_string(),
        71..=76 => "B-".to_string(),
        64..=70 => "C+".to_string(),
        57..=63 => "C".to_string(),
        50..=56 => "C-".to_string(),
        40..=49 => "D".to_string(),
        _ => "F".to_string(),
    }
}

pub fn grade_function(func: &crate::metrics::FunctionMetrics) -> FunctionGrade {
    let mut score: f64 = 100.0;
    let mut issues = Vec::new();
    let mut strengths = Vec::new();

    if func.line_count > 100 {
        score -= 40.0;
        issues.push(format!("{} lines long, extremely hard to maintain", func.line_count));
    } else if func.line_count > 50 {
        score -= 20.0;
        issues.push(format!("{} lines long, consider splitting", func.line_count));
    } else if func.line_count > 30 {
        score -= 10.0;
    } else if func.line_count <= 20 {
        strengths.push("Compact and focused".to_string());
    }

    if func.cyclomatic_complexity > 20 {
        score -= 30.0;
        issues.push(format!("Cyclomatic complexity {} (extremely high)", func.cyclomatic_complexity));
    } else if func.cyclomatic_complexity > 10 {
        score -= 15.0;
        issues.push(format!("Cyclomatic complexity {} (high)", func.cyclomatic_complexity));
    } else if func.cyclomatic_complexity <= 5 {
        strengths.push("Low complexity, easy to test".to_string());
    }

    if func.cognitive_complexity > 15 {
        score -= 25.0;
        issues.push(format!("Cognitive complexity {} (hard to understand)", func.cognitive_complexity));
    } else if func.cognitive_complexity > 8 {
        score -= 10.0;
    }

    if func.param_count > 7 {
        score -= 20.0;
        issues.push(format!("{} parameters (use a config table instead)", func.param_count));
    } else if func.param_count > 4 {
        score -= 10.0;
    } else if func.param_count <= 3 {
        strengths.push("Clean parameter signature".to_string());
    }

    if func.max_nesting > 6 {
        score -= 20.0;
        issues.push(format!("Nesting depth {} (deeply nested)", func.max_nesting));
    } else if func.max_nesting > 4 {
        score -= 10.0;
    }

    if func.has_error_handling {
        score += 5.0;
        strengths.push("Has error handling".to_string());
    }

    score = score.clamp(0.0, 100.0);
    let grade = score_to_grade(score);

    FunctionGrade {
        name: func.name.clone(),
        line: func.line,
        grade,
        score,
        complexity: func.cyclomatic_complexity,
        cognitive: func.cognitive_complexity,
        lines: func.line_count,
        issues,
        strengths,
    }
}
