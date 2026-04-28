# ChurnLens

ChurnLens is a high-performance static analysis tool designed to identify hotspots in TypeScript and JavaScript repositories. It correlates code complexity—derived from Abstract Syntax Tree (AST) traversal—with historical churn metrics extracted from Git metadata.

## Technical Overview

The engine is built in Rust to ensure memory safety and computational efficiency during the analysis of large-scale monorepos. 

* **AST Parsing:** Utilizes `tree-sitter` for robust, incremental parsing of TypeScript and JavaScript source files.
* **Git Mining:** Implements a single-pass `RevWalk` strategy via `libgit2` bindings. By traversing the commit graph once and caching OIDs, the tool avoids the $O(N \times M)$ overhead associated with per-file history queries.
* **Concurrency:** Leverages the `Rayon` data-parallelism library to distribute file parsing and metric aggregation across all available CPU cores.

## Installation

Compile from source using the Cargo package manager:

```bash
cargo build --release
```

The resulting binary will be located at `./target/release/churnlens`.

## Usage

ChurnLens operates as a CLI tool. It requires a path to a local Git repository.

```bash
# Basic analysis with JSON output
./churnlens /path/to/repo --output report.json

# Filtered analysis limited to the src directory
./churnlens /path/to/repo/src --limit 50

# Sort by specific telemetry
./churnlens . --sort churn_score
./churnlens . --sort cyclomatic_complexity
```

### CLI Arguments

| Argument | Description | Default |
| :--- | :--- | :--- |
| `path` | Path to the target directory or repository. | `.` |
| `--output` | Path to save the generated JSON report. | `stdout` |
| `--sort` | Metric used for ranking: `churn_score`, `complexity`, `modifications`. | `churn_score` |
| `--limit` | Maximum number of entries in the output. | `20` |

## Analysis Metrics

### Cyclomatic Complexity

The tool measures the number of linearly independent paths through a function's source code. This is calculated by identifying decision points (if-statements, loops, conditional expressions) within the AST.

### Risk Scoring (Churn)

The `churn_score` identifies unstable code sections by calculating the intersection of modification frequency and bug-fix density. The current heuristic is defined as:

$$churn\_score = \frac{m \times b}{a}$$

Where:
* $m$: Total times the file was modified in the analyzed history.
* $b$: Number of commits identified as bug fixes (via commit message heuristics).
* $a$: Total number of distinct authors contributing to the file.

## Project Architecture

The workspace is organized into a modular crate system to separate the core analysis engine from the CLI interface.

```text
ChurnLens/
├── crates/core/
│   ├── src/
│   │   ├── ast/        # Tree-sitter parsers and complexity visitors
│   │   ├── git/        # RevWalk logic and git2 integration
│   │   ├── metrics/    # Scoring algorithms and data structures
│   │   ├── error.rs    # Error propagation and recovery
│   │   ├── lib.rs      # Main engine entry point
│   │   └── main.rs     # CLI argument parsing and execution
│   └── Cargo.toml
├── Cargo.toml
└── README.md
```

## License

This project is licensed under the MIT License. See the `LICENSE` file for the full text.
