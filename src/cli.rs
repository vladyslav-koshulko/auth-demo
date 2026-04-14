use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "oauth-cli")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Login,
    Me,
    Logout,
    Cleanup,
}
