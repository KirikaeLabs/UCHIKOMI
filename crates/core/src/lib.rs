pub mod ast;
pub mod error;
pub mod git;
pub mod metrics;

use anyhow::Result;
use ast::parser::TypeScriptAnalyzer;
use git::GitAnalyzer;
use git2::Repository;
use ignore::WalkBuilder;
use metrics::{FunctionMetrics, Report, SummaryStats};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::borrow::Cow;

pub fn analyze_repository(
    repo_path: &Path,
    sort_by: &str,
    limit: Option<usize>,
) -> Result<Report<'static>> {
    log::info!("Analyzing repository: {}", repo_path.display());

    let repo = Repository::open(repo_path)?;
    let git_metrics = GitAnalyzer::get_all_file_metrics(&repo)?;

    let repo_path_abs = repo_path.canonicalize()?;
    
    // Use ignore crate for efficient walking
    let walker = WalkBuilder::new(&repo_path_abs)
        .standard_filters(true)
        .hidden(false) // we might want to see hidden files if they are JS/TS
        .build_parallel();

    let all_functions = Arc::new(Mutex::new(Vec::new()));

    walker.run(|| {
        let all_functions = Arc::clone(&all_functions);
        let repo_path_abs = repo_path_abs.clone();
        let git_metrics = &git_metrics;

        Box::new(move |entry| {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => return ignore::WalkState::Continue,
            };

            let path = entry.path();
            if !path.is_file() {
                return ignore::WalkState::Continue;
            }

            if let Some(ext) = path.extension() {
                if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
                    match TypeScriptAnalyzer::parse_file(path.to_str().unwrap()) {
                        Ok(mut functions) => {
                            if let Ok(rel_path) = path.strip_prefix(&repo_path_abs) {
                                let rel_path_str = rel_path.to_string_lossy().to_string();
                                if let Some(churn) = git_metrics.get(&rel_path_str) {
                                    for func in &mut functions {
                                        func.times_modified = churn.times_modified;
                                        func.bug_fix_commits = churn.bug_fix_commits;
                                        func.authors_count = churn.authors_count;
                                        func.churn_score = churn.churn_score;
                                        func.file = Cow::Owned(rel_path_str.clone());
                                    }
                                } else {
                                    for func in &mut functions {
                                        func.file = Cow::Owned(rel_path_str.clone());
                                    }
                                }
                            }
                            let mut all = all_functions.lock().unwrap();
                            all.extend(functions);
                        }
                        Err(e) => log::warn!("Failed to parse {}: {}", path.display(), e),
                    }
                }
            }

            ignore::WalkState::Continue
        })
    });

    let mut all_functions = Arc::try_unwrap(all_functions).unwrap().into_inner().unwrap();

    let mut file_metrics: HashMap<String, usize> = HashMap::new();
    for func in &all_functions {
        *file_metrics.entry(func.file.to_string()).or_insert(0) +=
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

fn sort_functions(functions: &mut [FunctionMetrics], sort_by: &str) {
    match sort_by {
        "churn_score" => functions.sort_by(|a, b| {
            b.churn_score.partial_cmp(&a.churn_score).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "cyclomatic_complexity" => functions.sort_by(|a, b| {
            b.cyclomatic_complexity.cmp(&a.cyclomatic_complexity)
        }),
        "cognitive_complexity" => functions.sort_by(|a, b| {
            b.cognitive_complexity.cmp(&a.cognitive_complexity)
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
