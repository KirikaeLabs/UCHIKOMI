pub mod ast;
pub mod error;
pub mod git;
pub mod metrics;

use anyhow::Result;
use ast::parser::TypeScriptAnalyzer;
use git::GitAnalyzer;
use metrics::{FunctionMetrics, Report, SummaryStats};
use std::collections::HashMap;
use std::path::Path;

pub fn analyze_repository(
    repo_path: &Path,
    sort_by: &str,
    limit: Option<usize>,
) -> Result<Report> {
    log::info!("Analyzing repository: {}", repo_path.display());

    let repo_path_str = repo_path.to_string_lossy().to_string();
    let mut all_functions = Vec::new();
    let mut file_metrics: HashMap<String, usize> = HashMap::new();

    walk_ts_files(repo_path, &mut |file_path| {
        log::debug!("Analyzing file: {}", file_path.display());

        match TypeScriptAnalyzer::parse_file(&file_path) {
            Ok(mut functions) => {
                for func in &mut functions {
                    if let Ok(churn) = GitAnalyzer::analyze_file(&repo_path_str, &func.file) {
                        func.times_modified = churn.times_modified;
                        func.bug_fix_commits = churn.bug_fix_commits;
                        func.authors_count = churn.authors_count;
                        func.churn_score = churn.churn_score;
                    }
                }

                all_functions.extend(functions);
            }
            Err(e) => log::warn!("Failed to parse {}: {}", file_path.display(), e),
        }
    })?;

    for func in &all_functions {
        *file_metrics.entry(func.file.clone()).or_insert(0) +=
            func.times_modified.max(1);
    }

    sort_functions(&mut all_functions, sort_by);

    if let Some(limit) = limit {
        all_functions.truncate(limit);
    }

    let summary = calculate_summary(&all_functions, &file_metrics);

    let report = Report {
        repository: repo_path_str,
        timestamp: chrono::Utc::now().to_rfc3339(),
        summary,
        functions: all_functions,
    };

    Ok(report)
}

fn walk_ts_files<F>(path: &Path, callback: &mut F) -> Result<()>
where
    F: FnMut(&Path),
{
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    if path.is_file() {
        if let Some(ext) = path.extension() {
            if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
                callback(path);
            }
        }
    } else if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with(".")
                    || name_str == "node_modules"
                    || name_str == "dist"
                    || name_str == "build"
                    || name_str == "target"
                {
                    continue;
                }
            }

            walk_ts_files(&path, callback)?;
        }
    }

    Ok(())
}

fn sort_functions(functions: &mut [FunctionMetrics], sort_by: &str) {
    match sort_by {
        "churn_score" => functions.sort_by(|a, b| {
            b.churn_score.partial_cmp(&a.churn_score).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "cyclomatic_complexity" => functions.sort_by(|a, b| {
            b.cyclomatic_complexity.cmp(&a.cyclomatic_complexity)
        }),
        "times_modified" => functions.sort_by(|a, b| {
            b.times_modified.cmp(&a.times_modified)
        }),
        "nesting_depth" => functions.sort_by(|a, b| {
            b.nesting_depth.cmp(&a.nesting_depth)
        }),
        "lines_of_code" => functions.sort_by(|a, b| {
            b.lines_of_code.cmp(&a.lines_of_code)
        }),
        _ => log::warn!("Unknown sort field: {}. Using churn_score.", sort_by),
    }
}

fn calculate_summary(
    functions: &[FunctionMetrics],
    file_metrics: &HashMap<String, usize>,
) -> SummaryStats {
    let total_functions = functions.len();
    let avg_complexity = if total_functions > 0 {
        functions.iter().map(|f| f.cyclomatic_complexity as f64).sum::<f64>() / total_functions as f64
    } else {
        0.0
    };

    let total_churn = functions.iter().map(|f| f.churn_score).sum();

    let mut most_churned: Vec<_> = file_metrics
        .iter()
        .map(|(file, churn)| (file.clone(), *churn))
        .collect();
    most_churned.sort_by(|a, b| b.1.cmp(&a.1));
    let most_churned_files = most_churned.iter().take(10).map(|(f, _)| f.clone()).collect();

    SummaryStats {
        total_functions,
        avg_complexity,
        total_churn,
        most_churned_files,
    }
}
