# ChurnLens: High-Performance Code Telemetry

ChurnLens is a specialized static analysis engine designed to quantify technical debt and stability risks in TypeScript and JavaScript repositories. It correlates Abstract Syntax Tree (AST) complexity with historical Git metadata to identify high-risk hotspots.

## Core Architecture

The engine is implemented in Rust, utilizing a two-pass analysis pipeline to provide globally normalized risk metrics across entire repositories.

### 1. Static Analysis Engine (AST)
* **Query-Based Parsing:** Utilizes `tree-sitter` with declarative S-expression queries. This leverages the underlying C-engine's optimized search for identifying function boundaries and complexity points.
* **Complexity Metrics:**
    * **Cyclomatic Complexity:** Measures linearly independent paths via AST decision points.
    * **Cognitive Complexity:** Implements a nesting-aware metric that penalizes deeply branched logic, providing a more accurate representation of maintainability.
    * **Nesting Depth:** Tracks the maximum depth of control structures to identify deeply nested, brittle logic.

### 2. Git Metadata Mining
* **Single-Pass RevWalk:** Performs a single traversal of the repository history ($O(\text{Commits} + \text{Files})$). Metadata is aggregated into a hash-mapped cache, eliminating the bottleneck of per-function history queries.
* **Refined Churn Calculation:** Implements a logarithmic weighting for bug fixes and contributor counts to dampen noise and highlight true hotspots.

### 3. Global Normalization and Risk Scoring
Unlike traditional tools that provide raw numbers, ChurnLens performs a **global statistical pass** before reporting:
* **Outlier Protection:** Uses a capped normalization strategy. If a metric's maximum is an extreme outlier (>3x the 95th percentile), the denominator is capped at the 99th percentile to prevent "God Functions" from squashing the risk scores of other code.
* **Percentile Ranking:** Every function is assigned a percentile rank for Risk, Churn, and Cognitive Complexity relative to the rest of the codebase.
* **Exponential Penalties:** Applies non-linear penalties for high nesting depth ( $depth > 3$ ) to account for the exponential increase in cognitive load.

## Risk Scoring Model

The `final_score` represents the total technical risk of a function:

$$Risk = BaseScore \times (1.0 + (\frac{depth}{4})^2 \times 0.20)$$

### Base Score Weights
| Metric | Weight | Description |
| :--- | :--- | :--- |
| **Cognitive** | 0.35 | Logical density and nesting. |
| **Churn** | 0.30 | Refined historical volatility. |
| **Cyclomatic** | 0.15 | Structural branching paths. |
| **LOC** | 0.10 | Raw surface area. |
| **Authors** | 0.10 | Fragmentation of ownership. |

### Refined Churn Formula
$$churn\_score = (m + (b \times 2)) \times \log_{10}(a + 1)$$
* **m**: Modification frequency.
* **b**: Bug-fix commits.
* **a**: Contributor count.

## Installation

Build the optimized binary from the workspace root:

```bash
cargo build --release
```

## Usage

```bash
# Analyze repository and output full JSON report
./target/release/churnlens [PATH] > report.json
```

### Output Format
The tool produces a comprehensive JSON report containing:
* **Summary Stats**: Total functions, global max values, and p95 distributions.
* **Function Telemetry**: 
    * `id`: Stable identifier (`file:name:line`).
    * `normalized`: Metrics scaled 0.0–1.0 with outlier protection.
    * `risk`: Base score, nesting penalty, and final score.
    * `percentile`: Global rank (0–100) for risk, churn, and complexity.

## License

MIT License. See `LICENSE` for details.
