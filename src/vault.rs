use std::path::{Path, PathBuf};

use crate::{Error, Result, global};


pub trait Vault {
    fn get_key(&self, key: &str) -> Result<String>;
    fn set_key(&mut self, key: &str, value: String) -> Result<()>;
}

const SIMPLE_VAULT_DIR: &str = "vault";

// SimpleVault stores all passwords in separate files
pub struct SimpleVault {
    vault_dir: PathBuf,
}

impl SimpleVault {
    /// Creates a new SimpleVault with the specified store. This may
    /// overwrite an existing vault.
    pub fn create(store: Option<String>) -> Result<SimpleVault> {
        let vault_dir = SimpleVault::get_path_or_default(store)?;

        if !vault_dir.as_path().exists() {
            std::fs::create_dir(vault_dir.as_path())?;
        }

        Ok(SimpleVault { vault_dir })
    }

    pub fn load(store: Option<String>) -> Result<SimpleVault> {
        let vault_dir = SimpleVault::get_path_or_default(store)?;
        Ok(SimpleVault { vault_dir })
    }

    /// Method to get a PathBuf to Some(String), or the default dir if None
    fn get_path_or_default(path: Option<String>) -> Result<PathBuf> {
        Ok(path.map(|p| PathBuf::from(p)).unwrap_or(SimpleVault::default_vault_path()?))
    }

    /// Returns a PathBuf representing the location of the default
    /// store location.
    fn default_vault_path() -> Result<PathBuf> {
        let vault_file = home::home_dir().ok_or(Error::NoHomeDir)?
            .join(Path::new(global::YAP_DIR))
            .join(Path::new(SIMPLE_VAULT_DIR));

        Ok(vault_file)
    }
}

impl Vault for SimpleVault {
    fn get_key(&self, key: &str) -> Result<String> {
        let p = self.vault_dir.join(Path::new(key));
        if !p.as_path().exists() {
            Err(Error::PasswordNotFound { name: key.to_string() })
        } else {

            // TODO decrypt

            Ok(std::fs::read_to_string(p.as_path())?)
        }
    }

    fn set_key(&mut self, key: &str, value: String) -> Result<()> {
        let p = self.vault_dir.join(Path::new(key));

        // TODO encrypt

        Ok(std::fs::write(p.as_path(), value)?)
    }
}