use clap::{Parser, Subcommand};

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
    #[arg(long, value_name = "finding-category")]
    pub category: String,
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
