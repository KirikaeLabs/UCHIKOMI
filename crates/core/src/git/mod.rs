use crate::metrics::ChurnMetrics;
use anyhow::Result;
use git2::Repository;
use std::collections::HashSet;

pub struct GitAnalyzer;

impl GitAnalyzer {
    pub fn analyze_file(repo_path: &str, file_path: &str) -> Result<ChurnMetrics> {
        let repo = Repository::open(repo_path)?;

        let mut times_modified = 0;
        let mut bug_fix_commits = 0;
        let mut authors = HashSet::new();

        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;

            if Self::file_modified_in_commit(&repo, &commit, file_path)? {
                times_modified += 1;

                if let Some(author_name) = commit.author().name() {
                    authors.insert(author_name.to_string());
                }

                if let Some(message) = commit.message() {
                    let lower_msg = message.to_lowercase();
                    if lower_msg.contains("fix") || lower_msg.contains("bug") {
                        bug_fix_commits += 1;
                    }
                }
            }
        }

        let authors_count = authors.len();
        let churn_score = if authors_count > 0 {
            (times_modified as f64 * bug_fix_commits as f64) / authors_count as f64
        } else {
            times_modified as f64
        };

        Ok(ChurnMetrics {
            times_modified,
            bug_fix_commits,
            authors_count,
            churn_score,
        })
    }

    fn file_modified_in_commit(
        repo: &Repository,
        commit: &git2::Commit,
        file_path: &str,
    ) -> Result<bool> {
        let tree = commit.tree()?;

        let parent_tree = if commit.parent_count() > 0 {
            commit.parent(0)?.tree()?
        } else {
            return Ok(true);
        };

        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

        for delta in diff.deltas() {
            if let Some(path) = delta.new_file().path() {
                if path.to_string_lossy().contains(file_path) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
