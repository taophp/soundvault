//! Configuration for SoundVault

use crate::error::{Result, VaultError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for SoundVault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Path to the library directory
    pub library_path: PathBuf,

    /// Path to the database file
    pub database_path: PathBuf,

    /// Freesound API key
    pub freesound_api_key: Option<String>,

    /// Default cache behavior for downloaded sounds
    pub cache_downloaded_sounds: bool,
}

impl VaultConfig {
    /// Create a new configuration with default values
    ///
    /// # Examples
    ///
    /// ```
    /// use soundvault::VaultConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = VaultConfig::new(
    ///     PathBuf::from("./sounds"),
    ///     Some("my_api_key".to_string())
    /// );
    ///
    /// assert_eq!(config.library_path, PathBuf::from("./sounds"));
    /// assert_eq!(config.freesound_api_key, Some("my_api_key".to_string()));
    /// assert!(config.cache_downloaded_sounds);
    /// ```
    pub fn new(library_path: PathBuf, freesound_api_key: Option<String>) -> Self {
        let db_path = library_path.join("soundvault.db");

        Self {
            library_path,
            database_path: db_path,
            freesound_api_key,
            cache_downloaded_sounds: true,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check if the library path exists or can be created
        if !self.library_path.exists() {
            return Err(VaultError::Config(format!(
                "Library path does not exist: {:?}",
                self.library_path
            )));
        }

        // Check if the library path is a directory
        if !self.library_path.is_dir() {
            return Err(VaultError::Config(format!(
                "Library path is not a directory: {:?}",
                self.library_path
            )));
        }

        Ok(())
    }
}
