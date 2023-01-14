pub mod error;
pub mod config;
pub mod vault;

mod global;
// mod crypto;

use std::path::Path;

pub use config::ConfigCommand;
pub use error::{Error, Result};

pub trait ExecutableCommand {
    fn execute(self) -> std::result::Result<String, String>;
}

/// Ensures that required directories and files exist
pub fn init() -> Result<()> {
    init_dir(global::YAP_DIR)
}

/// Initializes the directory for yap, inside the user's home directory.
fn init_dir(dir: &str) -> Result<()> {
    let yap_path = home::home_dir().ok_or(Error::NoHomeDir)?.join(Path::new(dir));

    if !yap_path.as_path().exists() {
        std::fs::create_dir(yap_path.as_path())?;
    }

    config::init()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::init_dir;

    #[test]
    fn init_creates_directories() {
        let yap_test = ".yap_test";
        let p = home::home_dir().unwrap().join(yap_test);

        init_dir(yap_test).unwrap();

        assert!(p.exists());

        std::fs::remove_dir_all(p).unwrap()
    }
}
