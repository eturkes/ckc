use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use ckc_cli::{detect_all, load_claims, load_documents, pipeline, replay, verify_all};

/// Default run output directory, shared by every subcommand's `--out` (SPEC §25
/// names `runs/research`). The demo manifest's recorded command string is pinned
/// separately in `pipeline::DEMO_COMMAND` for cross-location hash stability.
const DEFAULT_OUT_DIR: &str = "runs/research";

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
        #[arg(long, default_value = DEFAULT_OUT_DIR)]
        out: PathBuf,
    },
    /// Run solver/proof checks and emit certificates
    Verify {
        /// Bundle path (Phase-0: examples/research_kernel)
        bundle: String,
        /// Output directory
        #[arg(long, default_value = DEFAULT_OUT_DIR)]
        out: PathBuf,
    },
    /// Detect logical incompatibilities and factual inconsistencies
    Conflicts {
        /// Bundle path (Phase-0: examples/research_kernel)
        bundle: String,
        /// Output directory
        #[arg(long, default_value = DEFAULT_OUT_DIR)]
        out: PathBuf,
    },
    /// Assemble the SPEC-21/23 bilingual report JSON
    Report {
        /// Bundle path (Phase-0: examples/research_kernel)
        bundle: String,
        /// Output directory
        #[arg(long, default_value = DEFAULT_OUT_DIR)]
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
        #[arg(long, default_value = DEFAULT_OUT_DIR)]
        out: PathBuf,
    },
    /// Replay a committed run manifest and compare hashes (SPEC 18)
    Replay {
        /// Committed run manifest path
        manifest: PathBuf,
        /// Output directory
        #[arg(long, default_value = DEFAULT_OUT_DIR)]
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
            let conflicts = detect_all(&bundle, &report);
            pipeline::run_conflicts(&conflicts, &out)?;
        }
        Command::Report { bundle, out } => {
            let bundle = pipeline::load_bundle(&bundle)?;
            let report = verify_all(&bundle);
            let conflicts = detect_all(&bundle, &report);
            pipeline::run_report(
                &bundle,
                &load_claims(),
                &load_documents(),
                &report,
                &conflicts,
                &out,
            )?;
        }
        Command::Demo {
            scenario,
            replay,
            out,
        } => {
            pipeline::run_demo(&scenario, replay, &out)?;
        }
        Command::Replay { manifest, out } => {
            replay::run_replay(&manifest, &out)?;
        }
    }
    Ok(())
}
