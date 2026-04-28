use crate::metrics::FunctionMetrics;
use tree_sitter::Node;

pub struct ComplexityVisitor<'a> {
    source: &'a str,
    file_path: &'a str,
    pub functions: Vec<FunctionMetrics>,
}

impl<'a> ComplexityVisitor<'a> {
    pub fn new(source: &'a str, file_path: &'a str) -> Self {
        Self {
            source,
            file_path,
            functions: Vec::new(),
        }
    }

    pub fn visit(&mut self, node: Node<'a>) {
        match node.kind() {
            "function_declaration" | "arrow_function" | "method_definition" | "function_expression" => {
                if let Some(metrics) = self.extract_function_metrics(node) {
                    self.functions.push(metrics);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit(child);
        }
    }

    fn extract_function_metrics(&self, node: Node<'a>) -> Option<FunctionMetrics> {
        let name = self.extract_function_name(node)?;
        let line = node.start_position().row as u32 + 1;
        let cyclomatic_complexity = self.calculate_complexity(node);
        let nesting_depth = self.calculate_nesting_depth(node);
        let lines_of_code = self.calculate_loc(node);

        Some(FunctionMetrics {
            name,
            file: self.file_path.to_string(),
            line,
            cyclomatic_complexity,
            nesting_depth,
            lines_of_code,
            times_modified: 0,
            bug_fix_commits: 0,
            authors_count: 0,
            churn_score: 0.0,
        })
    }

    fn extract_function_name(&self, node: Node<'a>) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => return Some(self.get_node_text(child)),
                _ => {}
            }
        }
        Some("<anonymous>".to_string())
    }

    fn calculate_complexity(&self, node: Node<'a>) -> u32 {
        let mut complexity = 1;
        self.count_complexity_nodes(node, &mut complexity);
        complexity
    }

    fn count_complexity_nodes(&self, node: Node<'a>, count: &mut u32) {
        match node.kind() {
            "if_statement"
            | "else_clause"
            | "for_statement"
            | "while_statement"
            | "do_statement"
            | "switch_statement"
            | "case_clause"
            | "catch_clause"
            | "ternary_expression" => *count += 1,
            "binary_expression" => {
                if let Some(op) = self.get_operator(node) {
                    if op == "||" || op == "&&" {
                        *count += 1;
                    }
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.count_complexity_nodes(child, count);
        }
    }

    fn calculate_nesting_depth(&self, node: Node<'a>) -> u32 {
        let mut max_depth = 0;
        self.measure_nesting(node, 0, &mut max_depth);
        max_depth
    }

    fn measure_nesting(&self, node: Node<'a>, current_depth: u32, max_depth: &mut u32) {
        match node.kind() {
            "if_statement"
            | "for_statement"
            | "while_statement"
            | "do_statement"
            | "switch_statement"
            | "try_statement"
            | "block" => {
                let new_depth = current_depth + 1;
                if new_depth > *max_depth {
                    *max_depth = new_depth;
                }
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.measure_nesting(child, new_depth, max_depth);
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.measure_nesting(child, current_depth, max_depth);
                }
            }
        }
    }

    fn calculate_loc(&self, node: Node<'a>) -> u32 {
        let start = node.start_position().row;
        let end = node.end_position().row;
        (end - start + 1) as u32
    }

    fn get_node_text(&self, node: Node<'a>) -> String {
        node.utf8_text(self.source.as_bytes())
            .unwrap_or("<unknown>")
            .to_string()
    }

    fn get_operator(&self, node: Node<'a>) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "binary_expression" || child.kind() == "update_expression" {
                return Some(self.get_node_text(child));
            }
        }
        None
    }
}
