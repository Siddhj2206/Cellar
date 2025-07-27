use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod launch;
mod runners;
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            name,
            exe,
            installer,
            interactive,
        } => {
            cli::commands::add_game(name, exe, installer, interactive)?;
        }
        Commands::Launch { name } => {
            cli::commands::launch_game(name).await?;
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
        Commands::Runners { command } => {
            cli::commands::handle_runners_command(command).await?;
        }
        Commands::Prefix { command } => {
            cli::commands::handle_prefix_command(command).await?;
        }
    }

    Ok(())
}
