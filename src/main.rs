use clap::{Args, Parser, Subcommand};
use mo_offline_range_query::{
    Dataset, metrics,
    workload::{self, Workload},
};
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(version, about = "Verified, measured offline distinct range queries")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compare naive, block Mo and alternating Mo; always verify every answer.
    Compare {
        #[command(flatten)]
        source: Source,
        #[arg(long)]
        block_size: Option<usize>,
        #[arg(long, default_value_t = 7)]
        repetitions: usize,
        /// Write the full report as JSON; stdout always shows a compact table.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Args)]
struct Source {
    /// JSON: {"array": [integers], "queries": [{"left": 0, "right": 9}]}.
    #[arg(long, conflicts_with_all = ["n", "queries", "seed", "workload"])]
    input: Option<PathBuf>,
    #[arg(long)]
    n: Option<usize>,
    #[arg(long)]
    queries: Option<usize>,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, value_enum)]
    workload: Option<Workload>,
}

impl Source {
    fn load(&self) -> Result<Dataset, String> {
        let data = if let Some(path) = &self.input {
            serde_json::from_str(
                &fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
            )
            .map_err(|e| format!("invalid input JSON: {e}"))?
        } else {
            workload::generate(
                self.n.unwrap_or(1000),
                self.queries.unwrap_or(1000),
                self.seed.unwrap_or(42),
                self.workload.unwrap_or(Workload::Random),
            )?
        };
        data.validate()?;
        if data.array.len() > 10_000 || data.queries.len() > 2_000 {
            return Err("CLI limit: 10000 elements and 2000 queries".into());
        }
        Ok(data)
    }
}

fn run() -> Result<(), String> {
    match Cli::parse().command {
        Command::Compare {
            source,
            block_size,
            repetitions,
            output,
        } => {
            let data = source.load()?;
            let report = metrics::compare(&data, block_size, repetitions)?;
            println!(
                "n={} queries={} repetitions={} verified=true",
                report.n, report.queries, report.repetitions
            );
            if report.debug_build {
                eprintln!("Debug build: use cargo run --release for runtime comparisons.");
            }
            println!(
                "{:<14} {:>12} {:>12} {:>12} {:>12}",
                "strategy", "moves", "scanned", "buffers B", "median ms"
            );
            for m in &report.measurements {
                let cell = |v: Option<u64>| v.map_or_else(|| "n/a".into(), |v| v.to_string());
                println!(
                    "{:<14} {:>12} {:>12} {:>12} {:>12.3}",
                    m.strategy,
                    cell(m.pointer_moves),
                    cell(m.elements_scanned),
                    cell(m.buffer_capacity_bytes.map(|v| v as u64)),
                    m.timings_median.total_ns as f64 / 1e6
                );
            }
            println!("{}", report.memory_note);
            if let Some(path) = output {
                fs::write(
                    &path,
                    serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
                )
                .map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
