//! `ckc` binary entry: thin I/O wrapper over [`ckc_cli::run_cli`] — events
//! to stderr when no output directory took them, the command's stdout body
//! (the `ckc trace` chain, the `ckc replay` match report) and then the
//! single §4.4 result line to stdout, exit code from the outcome.

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exit = ckc_cli::run_cli(&args);
    if let Some(events) = &exit.streamed_events {
        let _ = std::io::stderr().write_all(events);
    }
    let mut stdout = std::io::stdout().lock();
    if let Some(body) = &exit.command_output {
        let _ = stdout.write_all(body);
    }
    let _ = stdout.write_all(&exit.result_line);
    let _ = stdout.flush();
    std::process::exit(exit.exit_code);
}
