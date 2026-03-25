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
    #[command(about = "List available themes")]
    List,
    #[command(about = "Show current theme")]
    Current,
    #[command(about = "Set the current theme")]
    Set(SetArgs),
}

#[derive(Args)]
pub struct SetArgs {
    pub theme: String,
    #[arg(long, help = "Preview changes")]
    pub dry_run: bool,
}
