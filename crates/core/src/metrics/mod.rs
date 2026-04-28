use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetrics {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub cyclomatic_complexity: u32,
    pub nesting_depth: u32,
    pub lines_of_code: u32,
    pub times_modified: usize,
    pub bug_fix_commits: usize,
    pub authors_count: usize,
    pub churn_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub repository: String,
    pub timestamp: String,
    pub summary: SummaryStats,
    pub functions: Vec<FunctionMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SummaryStats {
    pub total_functions: usize,
    pub avg_complexity: f64,
    pub total_churn: f64,
    pub most_churned_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ChurnMetrics {
    pub times_modified: usize,
    pub bug_fix_commits: usize,
    pub authors_count: usize,
    pub churn_score: f64,
}
