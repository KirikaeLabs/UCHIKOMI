use churnlens::analyze_repository;
use git2::{Repository, Signature};
use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn analyzes_small_git_repository() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let repo_path = temp_dir.path();
    let source_dir = repo_path.join("src");
    fs::create_dir(&source_dir).expect("source dir should be created");
    fs::write(
        source_dir.join("index.ts"),
        r#"
        function a() {
            if (x) {}
        }

        const b = () => {};
        "#,
    )
    .expect("source file should be written");

    let repo = Repository::init(repo_path).expect("repo should be initialized");
    let mut index = repo.index().expect("index should be available");
    index
        .add_path(std::path::Path::new("src/index.ts"))
        .expect("source file should be staged");
    index.write().expect("index should be written");
    let tree_id = index.write_tree().expect("tree should be written");
    let tree = repo.find_tree(tree_id).expect("tree should exist");
    let signature =
        Signature::now("Test User", "test@example.com").expect("signature should build");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "initial commit",
        &tree,
        &[],
    )
    .expect("commit should be created");

    let report = analyze_repository(
        repo_path,
        "churn_score",
        None,
        Arc::new(AtomicBool::new(false)),
    )
    .expect("repository should be analyzed");

    assert_eq!(report.schema_version, "0.1.0");
    assert_eq!(report.summary.total_functions, 2);
    assert_eq!(report.functions.len(), 2);
    assert!(report.analysis.commit.len() >= 40);

    let mut expected = report.functions.clone();
    expected.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.name.cmp(&b.name))
            .then(a.line.cmp(&b.line))
    });
    let actual = report.functions.clone();
    assert_eq!(
        expected
            .iter()
            .map(|function| &function.id)
            .collect::<Vec<_>>(),
        actual
            .iter()
            .map(|function| &function.id)
            .collect::<Vec<_>>()
    );

    assert!(report
        .functions
        .iter()
        .all(|function| function.risk.is_some()));

    let function_a = report
        .functions
        .iter()
        .find(|function| function.name == "a")
        .expect("function a should exist");
    assert_eq!(function_a.cyclomatic_complexity, 2);
    assert_eq!(function_a.cognitive_complexity, 1);
    assert_eq!(function_a.nesting_depth, 1);
    assert!(function_a.lines_of_code >= 3);
    assert!(function_a.lines_of_code <= 5);
    let normalized_a = function_a
        .normalized
        .as_ref()
        .expect("function a should have normalized metrics");
    assert_close(normalized_a.cyclomatic, 1.0);
    assert_close(normalized_a.cognitive, 1.0);
    assert_close(normalized_a.loc, 1.0);
    let percentile_a = function_a
        .percentile
        .as_ref()
        .expect("function a should have percentile metrics");
    assert_close(percentile_a.risk, 50.0);
    assert_close(percentile_a.cognitive, 50.0);

    let function_b = report
        .functions
        .iter()
        .find(|function| function.name == "b")
        .expect("function b should exist");
    assert_eq!(function_b.cyclomatic_complexity, 1);
    assert_eq!(function_b.cognitive_complexity, 0);
    assert_eq!(function_b.nesting_depth, 0);
    assert!(function_b.lines_of_code >= 1);
    assert!(function_b.lines_of_code <= 2);
    let normalized_b = function_b
        .normalized
        .as_ref()
        .expect("function b should have normalized metrics");
    assert_close(
        normalized_b.cyclomatic,
        std::f64::consts::LN_2 / 3.0_f64.ln(),
    );
    assert_close(normalized_b.cognitive, 0.0);
    assert_close(normalized_b.loc, 0.5);
    let percentile_b = function_b
        .percentile
        .as_ref()
        .expect("function b should have percentile metrics");
    assert_close(percentile_b.risk, 0.0);
    assert_close(percentile_b.cognitive, 0.0);

    let json = serde_json::to_string(&report).expect("report should serialize");
    assert!(json.contains("\"total_functions\":2"));
    assert!(json.contains("\"functions\""));
    assert!(json.contains("\"cyclomatic_complexity\""));
    assert!(json.contains("\"timestamp\""));

    let repeated_report = analyze_repository(
        repo_path,
        "churn_score",
        None,
        Arc::new(AtomicBool::new(false)),
    )
    .expect("repository should be analyzed repeatedly");
    assert_eq!(
        report.analysis.timestamp,
        repeated_report.analysis.timestamp
    );
    assert_ne!(report.analysis.timestamp, "1970-01-01T00:00:00+00:00");
}
