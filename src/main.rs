use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod api;
mod auth;
mod config;
mod qti;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get information about the current user
    Whoami,
    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCmd),
    /// Manage authentication
    #[command(subcommand)]
    Auth(AuthCmd),
    /// Convert a text file to a QTI zip file
    Convert { path: PathBuf },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Create a default configuration file
    Init,
    /// Get a configuration value
    Get { key: String },
    /// Set a configuration value
    Set { key: String, value: String },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Log in to Brightspace
    Login,
    /// Log out from Brightspace
    Logout,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCmd::Init => {
                let config_dir = dirs::config_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                    .join("brightspace");
                if !config_dir.exists() {
                    fs::create_dir_all(&config_dir)?;
                }
                let config_path = config_dir.join("brightspace-cli.yaml");

                let mut file = fs::File::create(&config_path)?;
                let default_config = serde_yaml::to_string(&config::Config::new())?;
                file.write_all(default_config.as_bytes())?;
                println!("Created default config file at: {:?}", config_path);
            }
            ConfigCmd::Get { key } => {
                let value = config::Config::get(key)?;
                println!("{}", value);
            }
            ConfigCmd::Set { key, value } => {
                config::Config::set(key, value)?;
                println!("Set {} to {}", key, value);
            }
        },
        Commands::Auth(auth_cmd) => match auth_cmd {
            AuthCmd::Login => {
                auth::login()?;
                println!("Credentials stored successfully.");
            }
            AuthCmd::Logout => {
                let username = config::Config::get("username")?;
                if username.is_empty() {
                    println!("Username is not set.");
                    return Ok(());
                }
                auth::delete_tokens(&username)?;
                println!("Credentials deleted successfully.");
            }
        },
        Commands::Whoami => {
            let config = config::Config::load()?;
            let access_token = auth::get_access_token(&config.username)?;
            let api = api::BrightspaceApi::new(config.base_url, access_token);
            let user = api.whoami().await?;
            println!("Display Name: {}", user.display_name);
            println!("Unique Identifier: {}", user.unique_identifier);
        }
        Commands::Convert { path } => {
            if let Err(e) = qti::convert_to_qti(&path) {
                eprintln!("Error converting quiz: {}", e);
            } else {
                println!("Conversion successful.");
            }
        }
    }

    Ok(())
}
