use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ckc", about = "Clinical Knowledge Compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a demo scenario
    Demo {
        /// Scenario name
        scenario: String,
        /// Replay and verify deterministic hashes
        #[arg(long)]
        replay: bool,
        /// Output directory
        #[arg(long)]
        out: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Demo {
            scenario,
            replay,
            out,
        } => {
            let out_dir = out.as_deref().unwrap_or("runs/default");
            println!("ckc demo: scenario={scenario} replay={replay} out={out_dir}");
            println!("phase-0 implementation pending");
            Ok(())
        }
    }
}
