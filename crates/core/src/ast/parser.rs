use crate::metrics::FunctionMetrics;
use std::fs;
use tree_sitter::Parser;
use tree_sitter_typescript::language_typescript;

use super::engine::ComplexityEngine;

pub struct TypeScriptAnalyzer;

impl TypeScriptAnalyzer {
    pub fn parse_file(path: &str) -> anyhow::Result<Vec<FunctionMetrics<'static>>> {
        let source = fs::read_to_string(path)?;
        // We use 'static here because we are reading to a String that will be dropped,
        // so we need to OWN the metrics or at least the strings in them.
        // Wait, the objective says zero-copy and efficient I/O.
        // To do zero-copy, the source MUST live as long as the metrics.
        // Let's change the API to return FunctionMetrics<'a> and let the caller manage the source.
        let mut parser = Parser::new();
        parser.set_language(language_typescript())?;
        
        let tree = parser.parse(&source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse {}", path))?;
            
        let engine = ComplexityEngine::new(&source, path);
        let functions = engine.analyze(tree.root_node());
        
        // Since 'source' is about to be dropped, we must convert Cow::Borrowed to Cow::Owned
        // to return them from this function if we read the file locally.
        // Alternatively, we can let the caller provide the source.
        
        Ok(functions.into_iter().map(|f| {
            FunctionMetrics {
                name: Cow::Owned(f.name.into_owned()),
                file: Cow::Owned(f.file.into_owned()),
                line: f.line,
                cyclomatic_complexity: f.cyclomatic_complexity,
                cognitive_complexity: f.cognitive_complexity,
                nesting_depth: f.nesting_depth,
                lines_of_code: f.lines_of_code,
                times_modified: f.times_modified,
                bug_fix_commits: f.bug_fix_commits,
                authors_count: f.authors_count,
                churn_score: f.churn_score,
            }
        }).collect())
    }

    pub fn analyze_source<'a>(source: &'a str, file_path: &'a str) -> anyhow::Result<Vec<FunctionMetrics<'a>>> {
        let mut parser = Parser::new();
        parser.set_language(language_typescript())?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse {}", file_path))?;

        let engine = ComplexityEngine::new(source, file_path);
        Ok(engine.analyze(tree.root_node()))
    }
}

use std::borrow::Cow;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        "#;

        let result = TypeScriptAnalyzer::analyze_source(source, "test.ts");
        assert!(result.is_ok());
        let functions = result.unwrap();
        assert!(!functions.is_empty());
    }

    #[test]
    fn test_parse_complex_function() {
        let source = r#"
        function validate(input: any): boolean {
            if (!input) return false;
            if (typeof input !== 'string') return false;
            if (input.length === 0) return false;
            if (input.length > 100) return false;
            return true;
        }
        "#;

        let result = TypeScriptAnalyzer::analyze_source(source, "test.ts");
        assert!(result.is_ok());
        let functions = result.unwrap();
        assert!(!functions.is_empty());
        assert!(functions[0].cyclomatic_complexity > 1);
        assert!(functions[0].cognitive_complexity > 0);
    }
}
