use std::collections::{HashMap, HashSet};

pub struct SuppressionMap {
    suppressions: HashMap<usize, HashSet<String>>,
}

impl SuppressionMap {
    pub fn from_source(source: &str) -> Self {
        let mut suppressions = HashMap::new();

        for (i, line) in source.lines().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if let Some(ids) = Self::parse_suppression(trimmed) {
                let target_line = line_num + 1;
                suppressions.entry(target_line)
                    .or_insert_with(HashSet::new)
                    .extend(ids);
            }

            if let Some(comment_start) = line.find("--") {
                let comment = line[comment_start..].trim();
                if comment != trimmed {
                    if let Some(ids) = Self::parse_suppression(comment) {
                        suppressions.entry(line_num)
                            .or_insert_with(HashSet::new)
                            .extend(ids);
                    }
                }
            }
        }

        Self { suppressions }
    }

    fn parse_suppression(comment: &str) -> Option<Vec<String>> {
        let stripped = comment.trim_start_matches('-').trim();
        if !stripped.starts_with("luau-grader:") { return None; }
        let after_prefix = stripped["luau-grader:".len()..].trim();
        if !after_prefix.starts_with("ignore") { return None; }
        let ids_str = after_prefix["ignore".len()..].trim();
        if ids_str.is_empty() { return None; }

        let ids: Vec<String> = ids_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if ids.is_empty() { None } else { Some(ids) }
    }

    pub fn is_suppressed(&self, line: usize, rule_id: &str) -> bool {
        self.suppressions
            .get(&line)
            .map(|ids| ids.contains(rule_id))
            .unwrap_or(false)
    }
}
