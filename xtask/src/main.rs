//! xtask - Development task runner for lambars
//!
//! Usage:
//!   cargo xtask bench-api --scenario <yaml> [options]

mod bench_api;
mod formal_catalog;
mod formal_coverage;
mod formal_patterns;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development task runner for lambars")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run API benchmarks with scenario configuration
    BenchApi(bench_api::BenchApiArgs),
    /// Validate the exhaustive formal-specification catalog.
    FormalCatalog(formal_catalog::FormalCatalogArgs),
    /// Validate the exhaustive formal coverage ledger.
    FormalCoverage(formal_coverage::FormalCoverageArgs),
    /// Detect repetitive implementation/proof shapes for macro extraction.
    FormalPatterns(formal_patterns::FormalPatternsArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::BenchApi(args) => bench_api::run(args),
        Commands::FormalCatalog(args) => formal_catalog::run(args),
        Commands::FormalCoverage(args) => formal_coverage::run(args),
        Commands::FormalPatterns(args) => formal_patterns::run(args),
    }
}
