use crate::{BwatchError, FindingCategory, poll};
use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "bwatch")]
#[command(about = "Outward tracker observation tool. Reads a finding category; emits a directive.")]
pub struct BwatchCli {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Poll(PollArgs),
    FindingCategories,
    Update,
    Init,
    Tail,
    Explain,
    Process,
}

#[derive(Debug, Clone, Eq, PartialEq, clap::Args)]
pub struct PollArgs {
    #[arg(long, value_name = "name-or-url")]
    pub source: Option<String>,
    #[arg(long, value_name = "path-or-name")]
    pub mission: Option<String>,
    #[arg(long, value_name = "path")]
    pub manifest: Option<String>,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub quiet: bool,
    #[arg(long, value_name = "text")]
    pub reason: Option<String>,
}

impl BwatchCli {
    pub fn run(self) -> Result<ExitCode, BwatchError> {
        match self.command {
            Cmd::Poll(args) => poll::run(args),
            Cmd::FindingCategories => {
                for category in FindingCategory::ALL {
                    println!("{category}");
                }
                Ok(ExitCode::SUCCESS)
            }
            Cmd::Update => placeholder("update"),
            Cmd::Init => placeholder("init"),
            Cmd::Tail => placeholder("tail"),
            Cmd::Explain => placeholder("explain"),
            Cmd::Process => placeholder("process"),
        }
    }
}

fn placeholder(command_name: &str) -> Result<ExitCode, BwatchError> {
    println!("bwatch {command_name} placeholder: behavior is deferred.");
    Ok(ExitCode::SUCCESS)
}
