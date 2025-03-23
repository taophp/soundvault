// src/lib.rs
//! SoundVault is a Rust library that allows you to organize your local sound library,
//! add your own audio files, search and download audio files from Freesound.org,
//! and provide seamless access for playback in your applications.

mod config;
mod error;
mod local;
mod models;
mod remote;
mod vault;

pub use config::VaultConfig;
pub use error::{Result, VaultError};
pub use models::{Collection, Sound, SoundMetadata, SoundSource};
pub use vault::SoundVault;

/// Version of the SoundVault library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
