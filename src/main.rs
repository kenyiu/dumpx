//! Process entrypoint for the `dumpx` CLI.

use std::process::ExitCode;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    dumpx::cli::run_from_env()
}
