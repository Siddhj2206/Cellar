use clap::Parser;
use anyhow::Result;

mod cli;
mod config;
mod utils;

use cli::commands::Commands;

#[derive(Parser)]
#[command(name = "cellar")]
#[command(about = "A wine prefix and game manager for Linux")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Add { name, exe, installer, interactive } => {
            cli::commands::add_game(name, exe, installer, interactive)?;
        }
        Commands::Launch { name } => {
            cli::commands::launch_game(name)?;
        }
        Commands::List => {
            cli::commands::list_games()?;
        }
        Commands::Remove { name } => {
            cli::commands::remove_game(name)?;
        }
        Commands::Info { name } => {
            cli::commands::show_game_info(name)?;
        }
        Commands::Status { name } => {
            cli::commands::show_status(name)?;
        }
    }
    
    Ok(())
}
