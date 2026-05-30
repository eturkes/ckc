use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use ckc_cli::{pipeline, verify_all};

#[derive(Parser)]
#[command(name = "ckc", about = "Clinical Knowledge Compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Emit the SPEC-14 target compiler portfolio
    Compile {
        /// Bundle path (Phase-0: examples/research_kernel)
        bundle: String,
        /// Output directory
        #[arg(long, default_value = "runs/research")]
        out: PathBuf,
    },
    /// Run solver/proof checks and emit certificates
    Verify {
        /// Bundle path (Phase-0: examples/research_kernel)
        bundle: String,
        /// Output directory
        #[arg(long, default_value = "runs/research")]
        out: PathBuf,
    },
    /// Detect logical incompatibilities and factual inconsistencies
    Conflicts {
        /// Bundle path (Phase-0: examples/research_kernel)
        bundle: String,
        /// Output directory
        #[arg(long, default_value = "runs/research")]
        out: PathBuf,
    },
    /// Run a demo scenario end to end
    Demo {
        /// Scenario name
        scenario: String,
        /// Replay and verify deterministic hashes
        #[arg(long)]
        replay: bool,
        /// Output directory
        #[arg(long, default_value = "runs/research")]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Compile { bundle, out } => {
            let bundle = pipeline::load_bundle(&bundle)?;
            pipeline::run_compile(&bundle, &out)?;
        }
        Command::Verify { bundle, out } => {
            let bundle = pipeline::load_bundle(&bundle)?;
            pipeline::run_verify(&bundle, &out)?;
        }
        Command::Conflicts { bundle, out } => {
            let bundle = pipeline::load_bundle(&bundle)?;
            let report = verify_all(&bundle);
            pipeline::run_conflicts(&bundle, &report, &out)?;
        }
        Command::Demo {
            scenario,
            replay,
            out,
        } => {
            pipeline::run_demo(&scenario, replay, &out)?;
        }
    }
    Ok(())
}
