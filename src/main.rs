use clap::{Parser, Subcommand};

use yap::{ExecutableCommand, ConfigCommand};
use yap::vault;

#[derive(Parser)]
#[command(about = "Yet Another Password Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Optional vault store to use. Useful if multiple vaults are in use.
    #[arg(short, long)]
    store: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Yap. This should only be used once, as it will remove any existing passwords in the specified store
    Init,

    /// Sync passwords with the remote repository
    Sync {
        /// Optional password store to sync if not default
        #[arg(short, long)]
        store: Option<String>
    },

    /// Set or view global settings
    Config {
        #[command(subcommand)]
        command: ConfigCommand
    },

    /// Get a password identified by 'name'
    Get {
        /// The name of the password
        name: String
    },

    /// Set a password to the given value. This will overwrite the password if it exists
    Set {
        /// The name of the password
        name: String,
        value: String,
    },

    /// Generate and store a password using the given name.
    Generate {
        /// The name of the password
        name: String
    },
}

impl ExecutableCommand for Cli {
    fn execute(self) -> Result<String, String> {
        match self.command {

            // Initialize the yap directory and the vaults
            Commands::Init => {
                yap::init()?;
                vault::create(self.store)?;
                Ok("Succesfully initialized Yap!".to_string())
            }

            // Execute the config subcommands
            Commands::Config { command } => command.execute(),

            // Sync the given store with a remote repository
            Commands::Sync { .. } => Ok("TODO".to_string()),

            // Get a password
            Commands::Get { name } => {
                let vault = vault::load(self.store)?;
                let pw = vault.get_key(name.as_str())?;
                Ok(pw)
            }

            // Set a password
            Commands::Set { name, value } => {
                let mut vault = vault::load(self.store)?;

                vault.set_key(name.as_str(), value)?;

                Ok("Successfully saved password".to_string())
            }

            // Generate and store a password
            Commands::Generate { .. } => Ok("TODO".to_string()),
        }
    }
}

fn main() {
    match Cli::parse().execute() {
        Ok(msg) => print!("{}", msg),
        Err(msg) => eprintln!("{}", msg)
    }
}
