pub mod ast;
pub mod error;
pub mod git;
pub mod metrics;

use anyhow::Result;
use ast::parser::TypeScriptAnalyzer;
use git::GitAnalyzer;
use git2::Repository;
use metrics::{FunctionMetrics, Report, SummaryStats};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn analyze_repository(
    repo_path: &Path,
    sort_by: &str,
    limit: Option<usize>,
) -> Result<Report> {
    log::info!("Analyzing repository: {}", repo_path.display());

    let repo = Repository::open(repo_path)?;
    let git_metrics = GitAnalyzer::get_all_file_metrics(&repo)?;

    let repo_path_abs = repo_path.canonicalize()?;
    let files = collect_ts_files(&repo_path_abs)?;

    let mut all_functions: Vec<FunctionMetrics> = files
        .par_iter()
        .flat_map(|file_path| {
            match TypeScriptAnalyzer::parse_file(file_path.to_str().unwrap()) {
                Ok(mut functions) => {
                    // Try to get relative path to match Git metrics
                    if let Ok(rel_path) = file_path.strip_prefix(&repo_path_abs) {
                        let rel_path_str = rel_path.to_string_lossy().to_string();
                        if let Some(churn) = git_metrics.get(&rel_path_str) {
                            for func in &mut functions {
                                func.times_modified = churn.times_modified;
                                func.bug_fix_commits = churn.bug_fix_commits;
                                func.authors_count = churn.authors_count;
                                func.churn_score = churn.churn_score;
                                func.file = rel_path_str.clone();
                            }
                        } else {
                            for func in &mut functions {
                                func.file = rel_path_str.clone();
                            }
                        }
                    }
                    functions
                }
                Err(e) => {
                    log::warn!("Failed to parse {}: {}", file_path.display(), e);
                    Vec::new()
                }
            }
        })
        .collect();

    let mut file_metrics: HashMap<String, usize> = HashMap::new();
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
        repository: repo_path_abs.to_string_lossy().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        summary,
        functions: all_functions,
    };

    Ok(report)
}

fn collect_ts_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_ts_files_recursive(path, &mut files)?;
    Ok(files)
}

fn walk_ts_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        if let Some(ext) = path.extension() {
            if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
                files.push(path.to_path_buf());
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

            walk_ts_files_recursive(&path, files)?;
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
