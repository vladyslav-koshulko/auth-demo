use clap::Parser;

mod cli;
mod models;
mod session;
mod server;


#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Login =>{
            println!("Starting server...");
            server::start_server().await;
        },
        cli::Commands::Me => println!("Me"),
        cli::Commands::Logout => println!("Logout"),
    }
}
