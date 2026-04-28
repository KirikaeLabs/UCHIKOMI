# ChurnLens

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ChurnLens** is a high-performance code analysis tool that measures code complexity, churn, and risk in TypeScript/JavaScript repositories.

## 🎯 Why ChurnLens?

Understand **why your code is slow and unstable**:
- 📊 **Cyclomatic Complexity**: Identify overly complex functions
- 🔄 **Code Churn**: Track which files change most frequently
- 🐛 **Bug Risk**: Correlate high churn with bug fixes
- ⚡ **Performance**: Analyze large repositories in seconds

## 🚀 Quick Start

### Installation

```bash
cargo build --release
```

### Usage

```bash
# Analyze current repository
./target/release/churnlens . --output report.json

# Analyze specific path
./target/release/churnlens ./src --output report.json

# Sort by different metrics
./target/release/churnlens . --sort churn_score --limit 30
./target/release/churnlens . --sort cyclomatic_complexity --limit 20
./target/release/churnlens . --sort times_modified

# Verbose output
./target/release/churnlens . -v
```

## 📊 Metrics Explained

### Cyclomatic Complexity
Measures the number of linearly independent paths through code.
- **1-3**: Simple, easy to maintain
- **4-7**: Moderate, acceptable
- **8-10**: Complex, hard to test
- **11+**: Very complex, refactor needed

### Code Churn
How frequently a file changes, combined with bug fixes.
```
churn_score = (times_modified × bug_fix_commits) / authors_count
```

## 🔧 Architecture

```
ChurnLens/
├── crates/core/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── error.rs
│   │   ├── metrics/
│   │   │   └── mod.rs
│   │   ├── ast/
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs
│   │   │   └── visitor.rs
│   │   └── git/
│   │       └── mod.rs
│   └── Cargo.toml
├── Cargo.toml
└── README.md
```

## 📝 License

MIT License - see LICENSE file for details
