//! Data models for the SoundVault library

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Source of a sound (local or remote)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundSource {
    /// Sound is stored in the local library
    Local,
    /// Sound is from Freesound.org
    Freesound,
}

/// Metadata for a sound
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundMetadata {
    /// Unique identifier for the sound
    pub id: String,

    /// Name of the sound
    pub name: String,

    /// Source of the sound
    pub source: SoundSource,

    /// Tags associated with the sound
    pub tags: Vec<String>,

    /// Description of the sound
    pub description: String,

    /// Duration in seconds
    pub duration: f32,

    /// License information
    pub license: String,

    /// Path to the file (for local sounds)
    pub path: Option<PathBuf>,

    /// Freesound ID (for remote sounds)
    pub freesound_id: Option<i32>,

    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

/// Sound object with metadata and content information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    /// Metadata for the sound
    pub metadata: SoundMetadata,

    /// Preview URL if available (local file URL or Freesound preview)
    pub preview_url: Option<String>,

    /// Whether the sound is available locally
    pub is_cached: bool,

    /// URL for downloading the sound (only for remote sounds)
    pub download_url: Option<String>,
}

/// Collection of sounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Unique identifier for the collection
    pub id: Uuid,

    /// Name of the collection
    pub name: String,

    /// Description of the collection
    pub description: String,

    /// Sound IDs in the collection
    pub sound_ids: Vec<String>,

    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

impl Collection {
    /// Create a new collection
    ///
    /// # Examples
    ///
    /// ```
    /// use soundvault::Collection;
    ///
    /// let collection = Collection::new("My Collection", "A collection of my favorite sounds");
    /// assert_eq!(collection.name, "My Collection");
    /// assert_eq!(collection.description, "A collection of my favorite sounds");
    /// assert!(collection.sound_ids.is_empty());
    /// ```
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            sound_ids: Vec::new(),
            custom: HashMap::new(),
        }
    }

    /// Add a sound to the collection
    ///
    /// # Examples
    ///
    /// ```
    /// use soundvault::Collection;
    ///
    /// let mut collection = Collection::new("My Collection", "A collection of my favorite sounds");
    /// collection.add_sound("sound-123");
    /// assert!(collection.contains_sound("sound-123"));
    /// ```
    pub fn add_sound(&mut self, sound_id: &str) {
        if !self.sound_ids.contains(&sound_id.to_string()) {
            self.sound_ids.push(sound_id.to_string());
        }
    }

    /// Remove a sound from the collection
    ///
    /// # Examples
    ///
    /// ```
    /// use soundvault::Collection;
    ///
    /// let mut collection = Collection::new("My Collection", "A collection of my favorite sounds");
    /// collection.add_sound("sound-123");
    /// assert!(collection.contains_sound("sound-123"));
    ///
    /// collection.remove_sound("sound-123");
    /// assert!(!collection.contains_sound("sound-123"));
    /// ```
    pub fn remove_sound(&mut self, sound_id: &str) {
        self.sound_ids.retain(|id| id != sound_id);
    }

    /// Check if the collection contains a sound
    pub fn contains_sound(&self, sound_id: &str) -> bool {
        self.sound_ids.contains(&sound_id.to_string())
    }

    /// Set a custom metadata value
    pub fn set_custom(&mut self, key: &str, value: &str) {
        self.custom.insert(key.to_string(), value.to_string());
    }

    /// Get a custom metadata value
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }
}

impl SoundMetadata {
    /// Set a custom metadata value
    pub fn set_custom(&mut self, key: &str, value: &str) {
        self.custom.insert(key.to_string(), value.to_string());
    }

    /// Get a custom metadata value
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }
}
