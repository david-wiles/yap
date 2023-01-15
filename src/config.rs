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
    pub fn init(p: PathBuf) -> Result<()> {
        if !p.as_path().exists() {
            std::fs::create_dir_all(p.as_path())?;
        }

        let f = File::create(p.join(CONFIG_FILE).as_path())?;
        serde_yaml::to_writer(f, &ConfigSettings::default())?;
        Ok(())
    }

    /// read will look for a config file in the specified directory and parse it into a Configuration
    pub fn read(dir: PathBuf) -> Result<Configuration> {
        let store = dir.join(CONFIG_FILE);
        let f = File::open(store.as_path())?;

        let settings: ConfigSettings = serde_yaml::from_reader(f)?;

        Ok(Configuration { settings, store })
    }

    /// Get the config value for the given key
    pub fn get_key(&self, key: SettingKey) -> &str {
        match key {
            SettingKey::RemoteURL => self.settings.remote_url.as_str(),
            SettingKey::Session => self.settings.session.as_str()
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
    pub fn save(&self) -> Result<()> {
        let f = File::create(self.store.as_path())?;
        Ok(serde_yaml::to_writer(f, &self.settings)?)
    }
}

/// Initializes a Configuration in the default directory
pub fn init() -> Result<()> {
    Configuration::init(get_default_path()?)
}

/// Reads a Configuration from the default directory
pub fn read() -> Result<Configuration> {
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
                let config = read()?;
                let setting_key = SettingKey::parse(key.as_str())
                    .ok_or(String::from(Error::BadConfigKey { key }))?;

                Ok(config.get_key(setting_key).to_string())
            }
            ConfigCommand::Set { key, value } => {
                let mut config = read()?;
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
    use std::path::Path;
    use crate::global::CONFIG_FILE;
    use crate::config::{Configuration, SettingKey};

    #[test]
    fn init_set_get_save_values() {
        let yap_test = Path::new(".yap_test");
        Configuration::init(yap_test.to_path_buf()).unwrap();
        assert!(yap_test.join(CONFIG_FILE).exists());

        let mut test_config = Configuration::read(yap_test.to_path_buf()).unwrap();

        let test_session = String::from("test session");
        test_config.set_key(SettingKey::Session, test_session.clone());
        assert_eq!(test_config.get_key(SettingKey::Session), test_session);

        let test_url = String::from("test remote url");
        test_config.set_key(SettingKey::RemoteURL, test_url.clone());
        assert_eq!(test_config.get_key(SettingKey::RemoteURL), test_url);

        test_config.save().unwrap();
        let test_config = Configuration::read(yap_test.to_path_buf()).unwrap();
        assert_eq!(test_config.get_key(SettingKey::Session), test_session);
        assert_eq!(test_config.get_key(SettingKey::RemoteURL), test_url);

        std::fs::remove_dir_all(yap_test).unwrap();
    }
}
