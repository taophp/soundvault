# SoundVault

SoundVault is a Rust library that allows you to organize your local sound library, add your own audio files, search and download audio files from Freesound.org, and provide seamless access for playback in your applications.

## Features

- **Unified library management** - Organize and search your local sound library with the same interface used for Freesound.org
- **Metadata management** - Edit and extend metadata for local sound files
- **Collections** - Group sounds into collections for easy access
- **Freesound integration** - Search, preview and download sounds from Freesound.org
- **Consistent API** - Use the same structures for both local and remote sounds

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
soundvault = "0.1.0"
```

Or use cargo:

```bash
cargo add soundvault
```

## Usage

### Basic Setup

```rust
use soundvault::{SoundVault, VaultConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration
    let config = VaultConfig::new(
        PathBuf::from("./my_sounds"),     // Library path
        Some("your_freesound_api_key".to_string()), // Optional API key
    );

    // Initialize the sound vault
    let vault = SoundVault::new(config).await?;

    Ok(())
}
```

### Working with Local Sounds

```rust
// Import a local sound file
let sound_id = vault.import_file("path/to/sound.wav", None).await?;

// Update metadata
vault.update_sound_metadata(sound_id, |metadata| {
    metadata.name = "Cool Sound Effect".to_string();
    metadata.tags = vec!["effect".to_string(), "cool".to_string()];
    metadata.description = "A really cool sound effect".to_string();
    metadata.set_custom("game_category", "ambient");
}).await?;

// Search for local sounds
let results = vault.search_local("cool effect", None).await?;
for sound in results {
    println!("Found: {} - {}", sound.metadata.name, sound.metadata.description);
}
```

### Working with Collections

```rust
// Create a new collection
let collection = Collection::new("Game Sounds", "Sounds for my game project");
let collection_id = vault.add_collection(collection).await?;

// Add sounds to the collection
vault.add_sound_to_collection(sound_id, collection_id).await?;

// Get all sounds in a collection
let sounds = vault.get_collection_sounds(collection_id).await?;
```

### Searching Freesound

```rust
// Search Freesound
let results = vault.search_freesound("piano", None).await?;
for sound in results {
    println!("Found on Freesound: {} by {}",
             sound.metadata.name,
             sound.metadata.get_custom("username").unwrap_or(&"unknown".to_string()));
}

// Download a sound to your local library
let freesound_id = 1234; // ID from Freesound
let local_sound_id = vault.download_sound(freesound_id).await?;
```

## Design Principles

- **Explicit source distinction** - SoundVault maintains a clear distinction between local and remote sounds to give applications full control
- **Uniform data structures** - The same structures are used for both sources to simplify application code
- **Extensible metadata** - Custom fields can be added to sounds and collections
- **Non-opinionated playback** - SoundVault focuses on organization and metadata, leaving playback to the application

## License

Licensed under the GNU Lesser General Public License v3.0 or later (LGPL-3.0-or-later).
