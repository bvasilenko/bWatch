#![forbid(unsafe_code)]

use bwatch::BwatchCli;
use clap::{Parser, error::ErrorKind};
use std::process::ExitCode;

fn main() -> ExitCode {
    match BwatchCli::try_parse() {
        Ok(cli) => run(cli),
        Err(error) => {
            let exit_code = clap_exit_code(&error);
            let _ = error.print();
            exit_code
        }
    }
}

fn run(cli: BwatchCli) -> ExitCode {
    match cli.run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            error.process_exit_code()
        }
    }
}

fn clap_exit_code(error: &clap::Error) -> ExitCode {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => ExitCode::SUCCESS,
        _ => bwatch::error::process_exit_code(bsuite_core::ExitCode::Usage),
    }
}
