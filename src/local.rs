//! Module for managing the local sound library

use crate::error::{Result, VaultError};
use crate::models::{Collection, Sound, SoundMetadata, SoundSource};
use sqlx::{Pool, Sqlite};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Manager for local sound files and metadata
pub struct LocalLibrary {
    /// Database connection pool
    db: Pool<Sqlite>,
    /// Path to the library directory
    library_path: PathBuf,
}

impl LocalLibrary {
    /// Create a new LocalLibrary
    ///
    /// # Arguments
    ///
    /// * `db` - SQLite connection pool
    /// * `library_path` - Path to the directory where sound files are stored
    pub async fn new(db: Pool<Sqlite>, library_path: PathBuf) -> Result<Self> {
        // Ensure the library directory exists
        if !library_path.exists() {
            std::fs::create_dir_all(&library_path)
                .map_err(|e| VaultError::FileSystem(format!("Failed to create library directory: {}", e)))?;
        }

        // Initialize database schema if needed
        Self::init_db_schema(&db).await?;

        Ok(Self { db, library_path })
    }

    /// Initialize the database schema if needed
    async fn init_db_schema(db: &Pool<Sqlite>) -> Result<()> {
        // Create sounds table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sounds (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                tags TEXT,
                duration REAL,
                license TEXT,
                path TEXT,
                freesound_id INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(db)
        .await?;

        // Create collections table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(db)
        .await?;

        // Create collection_sounds table for many-to-many relationship
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collection_sounds (
                collection_id TEXT,
                sound_id TEXT,
                PRIMARY KEY (collection_id, sound_id),
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (sound_id) REFERENCES sounds(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(db)
        .await?;

        // Create metadata table for custom metadata
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metadata (
                object_id TEXT NOT NULL,
                object_type TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT,
                PRIMARY KEY (object_id, object_type, key)
            )
            "#,
        )
        .execute(db)
        .await?;

        Ok(())
    }

    /// Import a sound file into the library
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the sound file to import
    /// * `metadata` - Optional metadata to set for the sound
    ///
    /// # Returns
    ///
    /// The ID of the imported sound
    pub async fn import_file<P: AsRef<Path>>(&self, source_path: P, metadata: Option<SoundMetadata>) -> Result<String> {
        let source_path = source_path.as_ref();

        // Check if file exists
        if !source_path.exists() {
            return Err(VaultError::FileSystem(format!(
                "Source file does not exist: {:?}",
                source_path
            )));
        }

        // Generate a unique ID for the sound
        let id = Uuid::new_v4().to_string();

        // Get file name from path
        let file_name = source_path.file_name().ok_or_else(|| {
            VaultError::FileSystem("Invalid source path".to_string())
        })?;

        // Create target path
        let target_path = self.library_path.join(&id).join(file_name);

        // Create directory for the sound
        std::fs::create_dir_all(target_path.parent().unwrap()).map_err(|e| {
            VaultError::FileSystem(format!("Failed to create directory: {}", e))
        })?;

        // Copy file to library
        std::fs::copy(source_path, &target_path).map_err(|e| {
            VaultError::FileSystem(format!("Failed to copy file: {}", e))
        })?;

        // Create metadata if not provided
        let metadata = if let Some(mut meta) = metadata {
            meta.id = id.clone();
            meta.path = Some(target_path);
            meta.source = SoundSource::Local;
            meta
        } else {
            // Extract basic metadata from file
            let name = file_name.to_string_lossy().to_string();
            SoundMetadata {
                id: id.clone(),
                name,
                source: SoundSource::Local,
                tags: Vec::new(),
                description: String::new(),
                duration: 0.0, // We'll need to implement audio file parsing to get this
                license: "Unknown".to_string(),
                path: Some(target_path),
                freesound_id: None,
                custom: Default::default(),
            }
        };

        // Insert into database
        self.save_metadata(&metadata).await?;

        Ok(id)
    }

    /// Save or update sound metadata in the database
    async fn save_metadata(&self, metadata: &SoundMetadata) -> Result<()> {
        // Convert tags to JSON string
        let tags_json = serde_json::to_string(&metadata.tags)
            .map_err(|e| VaultError::Json(e))?;

        // Insert or update sound record
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO sounds
            (id, name, description, tags, duration, license, path, freesound_id, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&metadata.id)
        .bind(&metadata.name)
        .bind(&metadata.description)
        .bind(tags_json)
        .bind(metadata.duration)
        .bind(&metadata.license)
        .bind(metadata.path.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(metadata.freesound_id)
        .execute(&self.db)
        .await?;

        // Update custom metadata
        for (key, value) in &metadata.custom {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO metadata
                (object_id, object_type, key, value)
                VALUES (?, 'sound', ?, ?)
                "#,
            )
            .bind(&metadata.id)
            .bind(key)
            .bind(value)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    /// Get a sound by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the sound to get
    ///
    /// # Returns
    ///
    /// The sound if found
    pub async fn get_sound(&self, id: &str) -> Result<Sound> {
        // Fetch basic sound data
        let sound_data = sqlx::query!(
            r#"
            SELECT id, name, description, tags, duration, license, path, freesound_id
            FROM sounds WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| VaultError::NotFound(format!("Sound not found: {}", id)))?;

        // Parse tags
        let tags: Vec<String> = if let Some(tags_str) = &sound_data.tags {
            serde_json::from_str(tags_str).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Fetch custom metadata
        let custom_meta = sqlx::query!(
            r#"
            SELECT key, value FROM metadata
            WHERE object_id = ? AND object_type = 'sound'
            "#,
            id
        )
        .fetch_all(&self.db)
        .await?;

        // Build custom metadata map
        let mut custom = std::collections::HashMap::new();
        for meta in custom_meta {
            if let (Some(key), Some(value)) = (meta.key, meta.value) {
                custom.insert(key, value);
            }
        }

        // Create path from string if available
        let path = sound_data.path.map(PathBuf::from);

        // Create metadata
        let metadata = SoundMetadata {
            id: sound_data.id,
            name: sound_data.name,
            source: SoundSource::Local,
            tags,
            description: sound_data.description.unwrap_or_default(),
            duration: sound_data.duration.unwrap_or_default(),
            license: sound_data.license.unwrap_or_default(),
            path,
            freesound_id: sound_data.freesound_id,
            custom,
        };

        // Generate preview URL (file:// URL for local playback)
        let preview_url = metadata.path.as_ref().map(|p| {
            format!("file://{}", p.to_string_lossy())
        });

        Ok(Sound {
            metadata,
            preview_url,
            is_cached: true,
            download_url: None,
        })
    }

    /// Search for sounds in local library
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    /// * `tags` - Optional tags to filter by
    ///
    /// # Returns
    ///
    /// List of matching sounds
    pub async fn search(&self, query: &str, tags: Option<&[&str]>) -> Result<Vec<Sound>> {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        // Add query condition if not empty
        if !query.is_empty() {
            conditions.push("(name LIKE ? OR description LIKE ?)");
            let query_pattern = format!("%{}%", query);
            params.push(query_pattern.clone());
            params.push(query_pattern);
        }

        // Add tag conditions if provided
        if let Some(tags) = tags {
            for tag in tags {
                conditions.push("tags LIKE ?");
                params.push(format!("%\"{}?\"%", tag));
            }
        }

        // Build the final query
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT id, name, description, tags, duration, license, path, freesound_id
            FROM sounds
            {}
            ORDER BY name ASC
            "#,
            where_clause
        );

        // Execute query and collect IDs
        let mut query = sqlx::query(&sql);
        for param in params {
            query = query.bind(param);
        }

        let rows = query.fetch_all(&self.db).await?;

        // Convert rows to IDs
        let mut ids = Vec::new();
        for row in rows {
            let id: &str = row.get(0);
            ids.push(id.to_string());
        }

        // Get full sound objects
        let mut sounds = Vec::new();
        for id in ids {
            sounds.push(self.get_sound(&id).await?);
        }

        Ok(sounds)
    }

    /// Update sound metadata
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the sound to update
    /// * `updater` - Function that updates the metadata
    pub async fn update_metadata<F>(&self, id: &str, updater: F) -> Result<()>
    where
        F: FnOnce(&mut SoundMetadata),
    {
        // Get current sound
        let mut sound = self.get_sound(id).await?;

        // Update metadata
        updater(&mut sound.metadata);

        // Save updated metadata
        self.save_metadata(&sound.metadata).await
    }

    /// Delete a sound from the library
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the sound to delete
    pub async fn delete_sound(&self, id: &str) -> Result<()> {
        // Get sound to find the file path
        let sound = self.get_sound(id).await?;

        // Delete file if it exists
        if let Some(path) = sound.metadata.path {
            if path.exists() {
                // Delete the parent directory (sound folder)
                let parent = path.parent().unwrap_or(&path);
                std::fs::remove_dir_all(parent).map_err(|e| {
                    VaultError::FileSystem(format!("Failed to delete sound directory: {}", e))
                })?;
            }
        }

        // Delete from database
        sqlx::query!("DELETE FROM sounds WHERE id = ?", id)
            .execute(&self.db)
            .await?;

        // Delete metadata
        sqlx::query!(
            "DELETE FROM metadata WHERE object_id = ? AND object_type = 'sound'",
            id
        )
        .execute(&self.db)
        .await?;

        // Delete from collections
        sqlx::query!(
            "DELETE FROM collection_sounds WHERE sound_id = ?",
            id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Create a new collection
    ///
    /// # Arguments
    ///
    /// * `collection` - Collection to create
    ///
    /// # Returns
    ///
    /// The ID of the created collection
    pub async fn add_collection(&self, collection: &Collection) -> Result<String> {
        // Convert collection ID to string
        let id = collection.id.to_string();

        // Insert collection
        sqlx::query!(
            r#"
            INSERT INTO collections (id, name, description)
            VALUES (?, ?, ?)
            "#,
            id,
            collection.name,
            collection.description,
        )
        .execute(&self.db)
        .await?;

        // Insert custom metadata
        for (key, value) in &collection.custom {
            sqlx::query!(
                r#"
                INSERT INTO metadata (object_id, object_type, key, value)
                VALUES (?, 'collection', ?, ?)
                "#,
                id,
                key,
                value,
            )
            .execute(&self.db)
            .await?;
        }

        // Insert sounds
        for sound_id in &collection.sound_ids {
            sqlx::query!(
                r#"
                INSERT OR IGNORE INTO collection_sounds (collection_id, sound_id)
                VALUES (?, ?)
                "#,
                id,
                sound_id,
            )
            .execute(&self.db)
            .await?;
        }

        Ok(id)
    }

    /// Get a collection by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the collection to get
    ///
    /// # Returns
    ///
    /// The collection if found
    pub async fn get_collection(&self, id: &str) -> Result<Collection> {
        // Fetch collection data
        let collection_data = sqlx::query!(
            r#"
            SELECT id, name, description
            FROM collections WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| VaultError::NotFound(format!("Collection not found: {}", id)))?;

        // Fetch sound IDs
        let sound_rows = sqlx::query!(
            r#"
            SELECT sound_id FROM collection_sounds WHERE collection_id = ?
            "#,
            id
        )
        .fetch_all(&self.db)
        .await?;

        let sound_ids: Vec<String> = sound_rows
            .into_iter()
            .filter_map(|row| row.sound_id)
            .collect();

        // Fetch custom metadata
        let custom_meta = sqlx::query!(
            r#"
            SELECT key, value FROM metadata
            WHERE object_id = ? AND object_type = 'collection'
            "#,
            id
        )
        .fetch_all(&self.db)
        .await?;

        // Build custom metadata map
        let mut custom = std::collections::HashMap::new();
        for meta in custom_meta {
            if let (Some(key), Some(value)) = (meta.key, meta.value) {
                custom.insert(key, value);
            }
        }

        // Parse UUID
        let uuid = uuid::Uuid::parse_str(&collection_data.id)
            .map_err(|_| VaultError::Database(sqlx::Error::RowNotFound))?;

        Ok(Collection {
            id: uuid,
            name: collection_data.name,
            description: collection_data.description.unwrap_or_default(),
            sound_ids,
            custom,
        })
    }

    /// Add a sound to a collection
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID of the sound to add
    /// * `collection_id` - ID of the collection to add to
    pub async fn add_sound_to_collection(&self, sound_id: &str, collection_id: &str) -> Result<()> {
        // Verify that both sound and collection exist
        self.get_sound(sound_id).await?;
        self.get_collection(collection_id).await?;

        // Add sound to collection
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO collection_sounds (collection_id, sound_id)
            VALUES (?, ?)
            "#,
            collection_id,
            sound_id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Remove a sound from a collection
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID of the sound to remove
    /// * `collection_id` - ID of the collection to remove from
    pub async fn remove_sound_from_collection(&self, sound_id: &str, collection_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM collection_sounds
            WHERE collection_id = ? AND sound_id = ?
            "#,
            collection_id,
            sound_id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// List all collections
    ///
    /// # Returns
    ///
    /// List of all collections
    pub async fn list_collections(&self) -> Result<Vec<Collection>> {
        // Fetch all collection IDs
        let collection_rows = sqlx::query!("SELECT id FROM collections")
            .fetch_all(&self.db)
            .await?;

        // Get each collection
        let mut collections = Vec::new();
        for row in collection_rows {
            if let Some(id) = row.id {
                collections.push(self.get_collection(&id).await?);
            }
        }

        Ok(collections)
    }

    /// Get all sounds in a collection
    ///
    /// # Arguments
    ///
    /// * `collection_id` - ID of the collection
    ///
    /// # Returns
    ///
    /// List of sounds in the collection
    pub async fn get_collection_sounds(&self, collection_id: &str) -> Result<Vec<Sound>> {
        // Get collection to verify it exists
        let collection = self.get_collection(collection_id).await?;

        // Get each sound
        let mut sounds = Vec::new();
        for sound_id in collection.sound_ids {
            sounds.push(self.get_sound(&sound_id).await?);
        }

        Ok(sounds)
    }

    /// List all sounds in the library
    ///
    /// # Returns
    ///
    /// List of all sounds
    pub async fn list_sounds(&self) -> Result<Vec<Sound>> {
        // Fetch all sound IDs
        let sound_rows = sqlx::query!("SELECT id FROM sounds")
            .fetch_all(&self.db)
            .await?;

        // Get each sound
        let mut sounds = Vec::new();
        for row in sound_rows {
            if let Some(id) = row.id {
                sounds.push(self.get_sound(&id).await?);
            }
        }

        Ok(sounds)
    }
}
