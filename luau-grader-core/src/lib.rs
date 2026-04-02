#![allow(
    clippy::collapsible_if,
    clippy::manual_range_contains,
    clippy::collapsible_match,
    clippy::redundant_closure
)]
pub mod analyzer;
pub mod config;
pub mod errors;
pub mod export;
pub mod fixer;
pub mod grade;
pub mod metrics;
pub mod report;
pub mod ruleset_config;
pub mod rulesets;
pub mod scorer;
pub mod batch;
pub mod workspace;
pub mod workspace_rules;
