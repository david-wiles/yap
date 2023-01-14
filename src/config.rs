use std::path::{Path, PathBuf};
use std::fs::File;

use serde::{Serialize, Deserialize};
use clap::Subcommand;

use crate::global::{YAP_DIR, CONFIG_FILE};
use crate::error::{Error, Result};
use crate::ExecutableCommand;


/// ConfigSettings are global settings for the program which should persist between command
/// invocations. These are saved to a file in the user's home directory and loaded every time that
/// yap is used.
#[derive(Serialize, Deserialize, Default)]
struct ConfigSettings {
    remote_url: String,
    session: String,
}

/// SettingKeys represent valid settings that can be updated by the user. These are parsed from a
/// string in the ExecutableCommand implementation for ConfigCommand.
pub enum SettingKey {
    RemoteURL,
    Session,
}

impl SettingKey {
    /// parse will create a SettingKey if the given string is valid. If the string does not
    /// correspond to a key, then a None option is returned.
    pub fn parse(setting: &str) -> Option<Self> {
        match setting {
            "remote_url" => Some(SettingKey::RemoteURL),
            "session" => Some(SettingKey::Session),
            _ => None
        }
    }
}

/// Configuration contains the config object (ConfigSettings) and a path to the location that the
/// configuration was loaded from. This enables multiple key vaults since each vault requires its
/// own settings, which may be different from the user's default.
pub struct Configuration {
    settings: ConfigSettings,
    store: PathBuf,
}

impl Configuration {
    /// Sets up required files and settings in the specified directory.
    pub(crate) fn init(p: PathBuf) -> Result<()> {
        if !p.as_path().exists() {
            std::fs::create_dir_all(p.as_path())?;
        }

        let f = File::create(p.join(CONFIG_FILE).as_path())?;
        serde_yaml::to_writer(f, &ConfigSettings::default())?;
        Ok(())
    }

    /// read will look for a config file in the specified directory and parse it into a Configuration
    pub(crate) fn read(dir: PathBuf) -> Result<Configuration> {
        let store = dir.join(CONFIG_FILE);
        let f = File::open(store.as_path())?;

        let settings: ConfigSettings = serde_yaml::from_reader(f)?;

        Ok(Configuration { settings, store })
    }

    /// Get the config value for the given key
    pub fn get_key(self, key: SettingKey) -> String {
        match key {
            SettingKey::RemoteURL => self.settings.remote_url,
            SettingKey::Session => self.settings.session
        }
    }

    /// Set the config value for the given key to the given value.
    pub fn set_key(&mut self, key: SettingKey, value: String) {
        match key {
            SettingKey::RemoteURL => self.settings.remote_url = value.clone(),
            SettingKey::Session => self.settings.session = value.clone(),
        }
    }

    /// Saves the Configuration into the default location.
    pub fn save(self) -> Result<()> {
        let f = File::create(self.store.as_path())?;
        Ok(serde_yaml::to_writer(f, &self.settings)?)
    }
}

/// Initializes a Configuration in the default directory
pub fn init() -> Result<()> {
    Configuration::init(get_default_path()?)
}

/// Reads a Configuration from the default directory
pub fn read_config() -> Result<Configuration> {
    Configuration::read(get_default_path()?)
}

/// Creates a PathBuf to the default config location
fn get_default_path() -> Result<PathBuf> {
    let home_dir = home::home_dir().ok_or(Error::NoHomeDir)?;
    let p = home_dir.join(Path::new(YAP_DIR));
    Ok(p)
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Get the value for the given setting
    Get {
        key: String
    },

    /// Set the value for the given setting
    Set {
        key: String,
        value: String,
    },
}

impl ExecutableCommand for ConfigCommand {
    fn execute(self) -> std::result::Result<String, String> {
        match self {
            ConfigCommand::Get { key } => {
                let config = read_config()?;
                let setting_key = SettingKey::parse(key.as_str())
                    .ok_or(String::from(Error::BadConfigKey { key }))?;

                Ok(config.get_key(setting_key))
            }
            ConfigCommand::Set { key, value } => {
                let mut config = read_config()?;
                let setting_key = SettingKey::parse(key.as_str())
                    .ok_or(String::from(Error::BadConfigKey { key }))?;

                config.set_key(setting_key, value);

                Ok(config.save().map(|_| "Successfully updated config.\n".to_string())?)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{config, global};
    use std::path::PathBuf;

    #[test]
    fn init_creates_config_file() {
        let yap_test = ".yap_test";
        let p = PathBuf::from(yap_test);
        if !p.exists() {
            std::fs::create_dir(p.as_path()).unwrap();
        }

        config::Configuration::init(PathBuf::from(yap_test)).unwrap();

        assert!(p.join(global::CONFIG_FILE).exists());

        std::fs::remove_dir_all(p.as_path()).unwrap();
    }

    #[test]
    fn read_returns_a_configuration() {
        let yap_test = ".yap_test";
        let p = PathBuf::from(yap_test);
        if !p.exists() {
            std::fs::create_dir(p.as_path()).unwrap();
        }

        let result = config::Configuration::read(PathBuf::from(yap_test));

        assert!(!result.is_err());

        std::fs::remove_dir_all(p.as_path()).unwrap();
    }

    #[test]
    fn get_key_returns_valid_value() {}

    #[test]
    fn get_key_returns_none_for_invalid_key() {}

    #[test]
    fn set_key_updates_value_for() {}

    #[test]
    fn set_key_returns_error_for_invalid_key() {}

    #[test]
    fn save_persists_values() {}
}
