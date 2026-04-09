use clap::Parser;

mod cli;
mod models;
mod session;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Login => println!("Login"),
        cli::Commands::Me => println!("Me"),
        cli::Commands::Logout => println!("Logout"),
    }
}
