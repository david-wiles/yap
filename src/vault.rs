use std::path::{Path, PathBuf};

use crate::{Error, Result, global};
use crate::crypto::Aes256GcmEngine;

// SimpleVault stores all passwords in separate files
pub struct SimpleVault {
    vault_dir: PathBuf,
    engine: Aes256GcmEngine,
}

impl SimpleVault {
    /// Creates a new SimpleVault with the specified store. This may
    /// overwrite an existing vault.
    pub(crate) fn create(vault_dir: PathBuf) -> Result<SimpleVault> {
        if !vault_dir.as_path().exists() {
            std::fs::create_dir(vault_dir.as_path())?;
        }

        // TODO remove testing only
        let pass = std::env::var("PASS").unwrap();
        let engine = Aes256GcmEngine::new(pass);

        Ok(SimpleVault { vault_dir, engine })
    }

    pub(crate) fn load(vault_dir: PathBuf) -> Result<SimpleVault> {
        // TODO remove testing only
        let pass = std::env::var("PASS").unwrap();
        let engine = Aes256GcmEngine::new(pass);

        Ok(SimpleVault { vault_dir, engine })
    }

    pub fn get_key(&self, key: &str) -> Result<String> {
        let p = self.vault_dir.join(Path::new(key));
        if !p.as_path().exists() {
            Err(Error::PasswordNotFound { name: key.to_string() })
        } else {
            let data = std::fs::read(p.as_path())?;
            let plaintext = self.engine.decrypt_bytes(data.as_slice())?;

            Ok(String::from_utf8(plaintext)?)
        }
    }

    pub fn set_key(&mut self, key: &str, value: String) -> Result<()> {
        let p = self.vault_dir.join(Path::new(key));
        let ciphertext = self.engine.encrypt_bytes(value.as_bytes())?;
        Ok(std::fs::write(p.as_path(), ciphertext)?)
    }
}

pub fn create(store: Option<String>) -> Result<SimpleVault> {
    let vault_dir = get_path_or_default(store)?;
    SimpleVault::create(vault_dir)
}

pub fn load(store: Option<String>) -> Result<SimpleVault> {
    let vault_dir = get_path_or_default(store)?;
    SimpleVault::load(vault_dir)
}

/// Method to get a PathBuf to Some(String), or the default dir if None
fn get_path_or_default(path: Option<String>) -> Result<PathBuf> {
    Ok(path.map(|p| PathBuf::from(p)).unwrap_or(default_vault_path()?))
}

/// Returns a PathBuf representing the location of the default
/// store location.
fn default_vault_path() -> Result<PathBuf> {
    let vault_file = home::home_dir().ok_or(Error::NoHomeDir)?
        .join(Path::new(global::YAP_DIR));

    Ok(vault_file)
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use crate::vault;

    #[test]
    fn create_and_load_simple_vault() {
        // TODO remove testing
        std::env::set_var("PASS", "asdf");
        let yap_test = String::from(".yap_test");
        std::fs::create_dir_all(Path::new(yap_test.as_str())).unwrap();

        let simple_vault = vault::create(Some(yap_test.clone()));
        assert!(simple_vault.is_ok());

        let simple_vault = vault::load(Some(yap_test));
        assert!(simple_vault.is_ok());
    }
}