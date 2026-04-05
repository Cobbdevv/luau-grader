pub mod beginner;
pub mod intermediate;
pub mod advanced;
pub mod front_page;
pub mod security;

use crate::analyzer::context::AnalysisContext;
use crate::config::Tier;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use crate::ruleset_config::{CustomRuleConfig, PatternConfig, RulesetConfig};
use full_moon::ast;
use full_moon::node::Node;
use serde::Serialize;

pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn category(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn tier(&self) -> &'static str;
    fn is_fixable(&self) -> bool { false }

    fn check_stmt(&self, _stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> { Vec::new() }
    fn check_expression(&self, _expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> { Vec::new() }
    fn check_function_body(&self, _body: &ast::FunctionBody, _ctx: &AnalysisContext) -> Vec<Diagnostic> { Vec::new() }
    fn finalize(&self, _ctx: &AnalysisContext) -> Vec<Diagnostic> { Vec::new() }
    fn fix(&self, _source: &str, _diagnostic: &Diagnostic) -> Option<Fix> { None }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleInfo {
    pub id: String,
    pub category: String,
    pub description: String,
    pub tier: String,
    pub severity: String,
    pub fixable: bool,
}

fn all_rules() -> Vec<Box<dyn Rule>> {
    all_rules_with_config(&RulesetConfig::default())
}

pub fn all_rules_with_config(config: &RulesetConfig) -> Vec<Box<dyn Rule>> {
    let max_lines: usize = config.get_param("I001", "max_lines").unwrap_or(50);
    let var_exceptions: Vec<String> = config.get_param("I002", "exceptions")
        .unwrap_or_else(|| vec!["i".into(), "j".into(), "k".into(), "_".into()]);
    let wfc_timeout: u64 = config.get_param("B005", "default_timeout").unwrap_or(5);
    let max_nesting: usize = config.get_param("I008", "max_nesting").unwrap_or(5);
    let max_file_lines: usize = config.get_param("F016", "max_lines").unwrap_or(500);

    let mut rules: Vec<Box<dyn Rule>> = vec![
        Box::new(beginner::DeprecatedWaitRule),
        Box::new(beginner::DeprecatedSpawnRule),
        Box::new(beginner::DeprecatedDelayRule),
        Box::new(beginner::InvokeClientRule),
        Box::new(beginner::WaitForChildTimeoutRule::new(wfc_timeout)),
        Box::new(beginner::InstanceNewParentArgRule),
        Box::new(beginner::DeprecatedLowercaseMethodRule),
        Box::new(beginner::DeprecatedTableFunctionRule),
        Box::new(beginner::GameDotWorkspaceRule),
        Box::new(beginner::ConstructorArgCountRule),
        Box::new(beginner::MethodArgCountRule),
        Box::new(beginner::StdLibArgCountRule),

        Box::new(intermediate::FunctionTooLongRule::new(max_lines)),
        Box::new(intermediate::SingleLetterVariableRule::new(var_exceptions)),
        Box::new(intermediate::GetServiceInLoopRule),
        Box::new(intermediate::NumericForWrongStepRule),
        Box::new(intermediate::EmptyIfBodyRule),
        Box::new(intermediate::RedundantTostringRule),
        Box::new(intermediate::RedundantTonumberRule),
        Box::new(intermediate::DeepNestingRule::new(max_nesting)),
        Box::new(intermediate::DebugPrintWarnRule),
        Box::new(intermediate::TableSortResultRule),
        Box::new(intermediate::TypeVsTypeofRule),
        Box::new(intermediate::Color3NewLargeValuesRule),
        Box::new(intermediate::SelfAssignmentRule),
        Box::new(intermediate::EmptyFunctionBodyRule),
        Box::new(intermediate::DuplicateTableKeyRule),
        Box::new(intermediate::HashLengthOnDictRule),
        Box::new(intermediate::WhileWaitDoRule),
        Box::new(intermediate::NilComparisonRule),
        Box::new(intermediate::NegatedConditionRule),
        Box::new(intermediate::MathHugeComparisonRule),
        Box::new(intermediate::InconsistentReturnRule),
        Box::new(intermediate::VariableShadowingRule),
        Box::new(intermediate::UnusedLocalRule),
        Box::new(intermediate::RepeatedAccessChainRule),
        Box::new(intermediate::VagueVariableNameRule),
        Box::new(intermediate::RedundantBooleanComparisonRule),
        Box::new(intermediate::TaskSpawnClosureWrappingRule),
        Box::new(intermediate::DuplicateGetServiceRule),

        Box::new(advanced::InstanceNewInLoopRule),
        Box::new(advanced::ConnectWithoutStoreRule),
        Box::new(advanced::StringConcatInLoopRule),
        Box::new(advanced::SetAsyncRule),
        Box::new(advanced::WhileTrueNoYieldRule),
        Box::new(advanced::TableInsertFrontInLoopRule),
        Box::new(advanced::ConnectInLoopRule),
        Box::new(advanced::PcallNoCheckRule),
        Box::new(advanced::CloneNotStoredRule),
        Box::new(advanced::DeprecatedLoadAnimationRule),
        Box::new(advanced::DeprecatedSetPrimaryPartCFrameRule),
        Box::new(advanced::DeprecatedMouseApiRule),
        Box::new(advanced::UnreachableCodeRule),
        Box::new(advanced::TableRemoveForwardLoopRule),
        Box::new(advanced::DeprecatedTickRule),
        Box::new(advanced::DeprecatedTweenSizeRule),
        Box::new(advanced::DebrisNegativeTimeRule),
        Box::new(advanced::StringFormatMismatchRule),
        Box::new(advanced::FindFirstChildInLoopRule),
        Box::new(advanced::GlobalWriteRule),
        Box::new(advanced::DeprecatedBodyMoverRule),
        Box::new(advanced::DirectHealthSetRule),
        Box::new(advanced::NestedPcallRule),
        Box::new(advanced::SetAsyncInPcallRule),
        Box::new(advanced::PcallErrorSwallowedRule),
        Box::new(advanced::ConnectWhenOnceSufficesRule),

        Box::new(front_page::NoStrictModeRule),
        Box::new(front_page::ParentNilWithoutDestroyRule),
        Box::new(front_page::RequireInLoopRule),
        Box::new(front_page::GetServiceWorkspaceRule),
        Box::new(front_page::FindFirstChildChainRule),
        Box::new(front_page::DeprecatedRemoveRule),
        Box::new(front_page::StringSubZeroIndexRule),
        Box::new(front_page::TaskWaitNegativeRule),
        Box::new(front_page::InstanceNewEmptyStringRule),
        Box::new(front_page::RenderSteppedOnServerRule),
        Box::new(front_page::WaitReturnValueRule),
        Box::new(front_page::ConnectWithNonFunctionRule),
        Box::new(front_page::TodoCommentRule),
        Box::new(front_page::LargeFileRule::new(max_file_lines)),
        Box::new(front_page::DeprecatedFilteringEnabledRule),
        Box::new(front_page::MissingTypeAnnotationRule),
        Box::new(front_page::HardcodedInstancePathRule),

        Box::new(security::UnvalidatedRemoteArgsRule),
        Box::new(security::NoRateLimitRule),
        Box::new(security::TrustClientPositionRule),
        Box::new(security::LoadstringUsageRule),
        Box::new(security::HttpNoPcallRule),
        Box::new(security::NoSessionLockRule),
        Box::new(security::GameDestroyRule),
        Box::new(security::NoSanityCheckRule),
    ];

    for custom in &config.custom_rules {
        rules.push(Box::new(CustomPatternRule::from_config(custom.clone())));
    }

    rules
}

pub fn list_all_rules() -> Vec<RuleInfo> {
    all_rules().iter().map(|r| RuleInfo {
        id: r.id().to_string(),
        category: r.category().to_string(),
        description: r.description().to_string(),
        tier: r.tier().to_string(),
        severity: format!("{:?}", r.severity()),
        fixable: r.is_fixable(),
    }).collect()
}

pub fn rules_for_tier(tier: Tier, disabled_rules: &[String]) -> Vec<Box<dyn Rule>> {
    rules_for_tier_with_config(tier, disabled_rules, &RulesetConfig::default())
}

pub fn rules_for_tier_with_config(tier: Tier, disabled_rules: &[String], config: &RulesetConfig) -> Vec<Box<dyn Rule>> {
    let included: &[&str] = match tier {
        Tier::Beginner => &["Beginner"],
        Tier::Intermediate => &["Beginner", "Intermediate"],
        Tier::Advanced => &["Beginner", "Intermediate", "Advanced"],
        Tier::FrontPage => &["Beginner", "Intermediate", "Advanced", "Front Page"],
    };

    all_rules_with_config(config)
        .into_iter()
        .filter(|r| included.contains(&r.tier()) && !disabled_rules.contains(&r.id().to_string()))
        .collect()
}

#[derive(Debug)]
pub struct CustomPatternRule {
    id: &'static str,
    description: &'static str,
    severity: Severity,
    category: &'static str,
    tier: &'static str,
    pattern: PatternConfig,
    message: &'static str,
    suggestion: Option<&'static str>,
}

impl CustomPatternRule {
    /// Converts a deserialized config into a rule instance.
    ///
    /// # Safety note on `Box::leak`
    /// Strings are intentionally leaked to `'static` because custom rules are
    /// loaded once at startup from `.luaugraderrc` and must satisfy the `Rule`
    /// trait's `&'static str` return types. This is a bounded, one-time cost.
    pub fn from_config(config: CustomRuleConfig) -> Self {
        let severity = match config.severity.as_str() {
            "Error" => Severity::Error,
            "Info" => Severity::Info,
            _ => Severity::Warning,
        };
        Self {
            id: Box::leak(config.id.into_boxed_str()),
            description: Box::leak(config.description.into_boxed_str()),
            severity,
            category: Box::leak(config.category.into_boxed_str()),
            tier: Box::leak(config.tier.into_boxed_str()),
            pattern: config.pattern,
            message: Box::leak(config.message.into_boxed_str()),
            suggestion: config.suggestion.map(|s| &*Box::leak(s.into_boxed_str())),
        }
    }
}

impl Rule for CustomPatternRule {
    fn id(&self) -> &'static str { self.id }
    fn severity(&self) -> Severity { self.severity }
    fn category(&self) -> &'static str { self.category }
    fn description(&self) -> &'static str { self.description }
    fn tier(&self) -> &'static str { self.tier }

    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt
            && self.matches_call(call) {
                return vec![self.make_diagnostic(call)];
        }
        Vec::new()
    }

    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr
            && self.matches_call(call) {
                return vec![self.make_diagnostic(call)];
        }
        Vec::new()
    }
}

impl CustomPatternRule {
    fn matches_call(&self, call: &ast::FunctionCall) -> bool {
        match &self.pattern {
            PatternConfig::FunctionCall { name } => {
                if let ast::Prefix::Name(prefix_name) = call.prefix() {
                    return prefix_name.token().to_string() == *name;
                }
                false
            }
            PatternConfig::MethodCall { name } => {
                for suffix in call.suffixes() {
                    if let ast::Suffix::Call(ast::Call::MethodCall(method)) = suffix
                        && method.name().token().to_string() == *name {
                            return true;
                    }
                }
                false
            }
        }
    }

    fn make_diagnostic(&self, call: &ast::FunctionCall) -> Diagnostic {
        Diagnostic {
            rule_id: self.id.to_string(),
            severity: self.severity,
            category: self.category.to_string(),
            message: self.message.to_string(),
            span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
            suggestion: self.suggestion.map(String::from),
            fixable: false,
        }
    }
}
