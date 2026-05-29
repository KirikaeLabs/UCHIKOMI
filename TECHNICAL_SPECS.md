# ChurnLens: Exhaustive Technical Specification & Architecture Manual

## 1. Executive Summary
ChurnLens is a high-performance, deterministic telemetry engine written in Rust. It serves as the **ground truth** for autonomous AI refactoring agents by correlating Abstract Syntax Tree (AST) complexity with historical change patterns (Git). It produces structured, normalized data designed for machine consumption, enabling AI agents to prioritize and execute code improvements through iterative cycles.

---

## 2. Core Architecture & Module Breakdown

### 2.1 Static Analysis Layer: AST Engine (`ast/`)
The engine uses **Tree-sitter** for multi-language parsing. It is designed to be extensible and safe.

#### 2.1.1 `LanguageSupport` Trait (`mod.rs`)
Defines the interface for language-specific analysis.
-   **Rust (`rust.rs`)**:
    -   Targets: `function_item`, `method_declaration`.
    -   Complexity: `if_expression`, `match_expression`, `for_expression`, `while_expression`, `match_arm`, and the `?` operator.
-   **TypeScript/JS (`typescript.rs`)**:
    -   Targets: `function_declaration`, `arrow_function`, `method_definition`.
    -   Complexity: `if`, `for`, `while`, `do`, `case`, `catch`, `&&`, `||`, `?`.
    -   Naming: Implements heuristics to resolve names for anonymous functions assigned to variables or properties.
-   **C (`c.rs`)**:
    -   Targets: `function_definition`.
    -   Complexity: Standard C control flow structures.

#### 2.1.2 `ComplexityEngine` (`engine.rs`)
A stack-based, non-recursive traversal engine.
-   **Function `analyze(root_node)`**: Walks the tree to compute metrics. It manages a `function_stack` to isolate metrics of nested functions from their parents.
-   **Body Quality Analysis**:
    -   **`body_hash`**: Stable XXHash3-128 hash of the function body.
    -   **Executable Statements**: Counts return, expression, declaration, assignment, call, and control flow nodes.
    -   **Hollow Check**: Detects functions with no executable statements.
        -   `hollow_kind`: `none`, `empty` (zero lines), or `comment_only`.
    -   **Documentation**: Analyzes leading RustDoc/JSDoc style comments.
        -   `documentation_quality`: `missing`, `sparse` (low chars/complexity ratio), or `adequate`.
    -   **Identifier Verbosity**: Average length of identifiers found within the function scope.
-   **Coupling Map**: `collect_call_names` identifies all unique function calls (Static Analysis) within the body.

---

### 2.2 Historical Analysis Layer: Git Analyzer (`git/mod.rs`)
Leverages `libgit2` for deep repository mining with a focus on function-level attribution.

#### 2.2.1 Mining Strategy
-   **Incremental Traversal**: Uses cached OIDs to process only new commits since the last analysis.
-   **Merge Handling**: Compares merge commits against all parents to accurately attribute changes.
-   **Rename Tracking**: Historical metrics are propagated through file renames to maintain continuity.
-   **Line-Level Attribution**: Maps `DiffHunk` line ranges to AST function ranges.

#### 2.2.2 Formulas & Churn Logic
-   **Refined Churn Formula**:
    `churn_score = (modifications + (bug_fixes * 2)) * log10(authors + 1)`
-   **Bug-Fix Detection**: Word-like token detection in commit messages (e.g., `fix`, `bug`, `issue`, `close`, `resolve`).
-   **Velocity**: Derived by comparing the 7-day modification rate against the 90-day rate.
    -   `accelerating` (ratio > 1.25), `cooling` (ratio < 0.75), or `stable`.

---

### 2.3 Persistence Layer: Cache System (`cache.rs`)
Ensures rapid feedback for iterative agents via `.churnlens/cache.bin`.
-   **Header**: Magic bytes `CHRN` (0x4348524E) + `CACHE_VERSION`.
-   **Invalidation Triggers**:
    -   Change in repository root, branch, or HEAD OID.
    -   Change in the `bug_fix_patterns` configuration.
    -   File content hash mismatch (AST results).

---

### 2.4 Orchestration & Scoring Layer (`lib.rs`)

#### 2.4.1 Global Normalization & Outlier Protection
To handle "God Functions" (extreme complexity/churn), ChurnLens applies:
-   **Capping**: If a maximum value is >3x the 95th percentile, the denominator is capped at the 99th percentile.
-   **Scaling**: All metrics are scaled [0.0, 1.0] using logarithmic scaling: `normalized = ln(1 + value) / ln(1 + cap)`.

#### 2.4.2 Risk Scoring
`FinalRisk = BaseScore * NestingPenalty * FanInMultiplier`
-   **Base Score Weights**: Cognitive (35%), Churn (30%), Cyclomatic (15%), LoC (10%), Authors (10%).
-   **Nesting Penalty**: `1.0 + (max_depth / 4)^2 * 0.20`.
-   **Fan-In Multiplier**: `1.0 + normalized_fan_in * 0.25`.
-   **Primary Driver**: The specific metric that contributed most to the Base Score.

#### 2.4.3 Project Statistics
-   **Bus Factor**: Minimum number of authors responsible for >50% of the total contributions.
-   **Tech Debt Density**: `(Total Cognitive + Total Cyclomatic) / Total LOC`.
-   **Dead Code**:
    -   `unreachable_private`: No callers, not exported.
    -   `unreachable_export`: Exported but unused internally.

---

## 3. Autonomous Refactoring Ecosystem

### 3.1 The Refactoring Loop
1.  **Analyze (ChurnLens)**: Produces the telemetry JSON.
2.  **Augment (Context AI)**: Isolates the function and its "Digital Brain" metadata (types, callee signatures).
3.  **Refactor (Agent)**: Targets the `primary_driver` while respecting `instability` and `reachability`.
4.  **Validate (Reviewers)**: Style and Correctness AIs approve the change.
5.  **Converge**: ChurnLens re-runs; `body_hash` change is verified, and risk percentile is recalculated.

---

## 4. JSON Schema Contract (Top-Level)

| Field | Description |
| :--- | :--- |
| `schema_version` | Current version of the JSON contract. |
| `summary` | Aggregated stats: `bus_factor`, `tech_debt_density`, `top_hotspots`, `dead_code`. |
| `quality` | Report health: `ast_hits`, `ast_misses`, `git_partial` flag, `warnings`. |
| `functions` | Array of function objects (Metrics, Churn, Risk, Percentile). |

---

## 5. Usage & Configuration

### CLI Flags
-   `--path`: Root directory (defaults to `.`).
-   `--sort`: `file`, `risk`, `churn_score`, `cognitive`, `cyclomatic`, `loc`.
-   `--limit`: Maximum number of functions in the report.

### `churnlens.toml`
```toml
[git]
# Custom regex for bug-fix identification
bug_fix_patterns = ["(?i)\\bfix(?:e[sd])?\\b", "JIRA-[0-9]+"]
```

---

## 6. Characteristics & Design Constraints
-   **Deterministic**: Output is identical for the same input state and version.
-   **Performance**: Uses `Rayon` for multi-threaded traversal and `memmap2` for large file reads.
-   **Machine-First**: No human-readable tables; strictly JSON data for automation pipelines.
-   **Best-Effort Static Analysis**: Static reachability and coupling do not resolve dynamic dispatch or reflection.
