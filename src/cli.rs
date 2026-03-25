use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "hue",
    version,
    about = "Sync themes across Ghostty, Helix, and Zed"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    List,
    Current,
    Set(SetArgs),
}

#[derive(Args)]
pub struct SetArgs {
    pub theme: String,
    #[arg(long, help = "Preview changes without writing files")]
    pub dry_run: bool,
}
