use anyhow::Result;
use churnlens_core::analyze_repository;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "churnlens")]
#[command(about = "Analyze code complexity and churn", long_about = None)]
struct Args {
    /// Path to repository
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output file (JSON)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Sort by field (churn_score, cyclomatic_complexity, times_modified, nesting_depth, lines_of_code)
    #[arg(short, long, default_value = "churn_score")]
    sort: String,

    /// Limit number of results
    #[arg(short, long)]
    limit: Option<usize>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    let report = analyze_repository(
        &args.path,
        &args.sort,
        args.limit,
    )?;

    if let Some(output_path) = args.output {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&output_path, json)?;
        println!("✅ Report saved to: {}", output_path.display());
    } else {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
    }

    Ok(())
}
