use crate::metrics::FunctionMetrics;
use std::fs;
use tree_sitter::{Language, Parser};
use tree_sitter_typescript::language_typescript;

use super::visitor::ComplexityVisitor;

pub struct TypeScriptAnalyzer;

impl TypeScriptAnalyzer {
    pub fn parse_file(path: &str) -> anyhow::Result<Vec<FunctionMetrics>> {
        let source = fs::read_to_string(path)?;
        Self::parse_source(&source, path)
    }

    pub fn parse_source(source: &str, file_path: &str) -> anyhow::Result<Vec<FunctionMetrics>> {
        let mut parser = Parser::new();
        let language = language_typescript();
        parser.set_language(language)?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse {}", file_path))?;

        let mut visitor = ComplexityVisitor::new(source, file_path);
        visitor.visit(tree.root_node());

        Ok(visitor.functions)
    }
}

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

        let result = TypeScriptAnalyzer::parse_source(source, "test.ts");
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

        let result = TypeScriptAnalyzer::parse_source(source, "test.ts");
        assert!(result.is_ok());
        let functions = result.unwrap();
        assert!(!functions.is_empty());
        assert!(functions[0].cyclomatic_complexity > 1);
    }
}
