//! Main module for SoundVault

use crate::config::VaultConfig;
use crate::error::{Result, VaultError};
use crate::local::LocalLibrary;
use crate::models::{Collection, Sound, SoundMetadata, SoundSource};
use crate::remote::FreesoundManager;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;

/// Main entry point for SoundVault functionality
pub struct SoundVault {
    /// Local library manager
    local: LocalLibrary,
    /// Freesound manager (optional)
    remote: Option<FreesoundManager>,
    /// Configuration
    config: VaultConfig,
}

impl SoundVault {
    /// Create a new SoundVault instance
    ///
    /// # Examples
    ///
    /// ```
    /// use soundvault::{SoundVault, VaultConfig};
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = VaultConfig::new(
    ///     PathBuf::from("./my_sounds"),
    ///     Some("your_freesound_api_key".to_string())
    /// );
    ///
    /// let vault = SoundVault::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: VaultConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Ensure the directory exists
        if !config.library_path.exists() {
            std::fs::create_dir_all(&config.library_path)
                .map_err(|e| VaultError::FileSystem(format!("Failed to create library directory: {}", e)))?;
        }

        // Connect to SQLite database
        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", config.database_path.display()))
            .await
            .map_err(|e| VaultError::Database(e))?;

        // Initialize local library
        let local = LocalLibrary::new(db, config.library_path.clone()).await?;

        // Initialize remote manager if API key is provided
        let remote = config.freesound_api_key.clone().map(|api_key| {
            FreesoundManager::new(api_key, config.library_path.clone())
        });

        Ok(Self {
            local,
            remote,
            config,
        })
    }

    // This will be filled with the actual implementation
}
