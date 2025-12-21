use crate::cli::OriginKind;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_origin: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_origin: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Origin {
    pub id: String,
    pub kind: OriginKind,
    pub url: String,
    pub token: String,
}

pub fn get_app_dir() -> Result<PathBuf> {
    let dir = if cfg!(target_os = "windows") {
        dirs::data_dir()
            .context("Failed to get AppData directory")?
            .join("nrelay")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".config")
            .join("nrelay")
    } else {
        dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".config")
            .join("nrelay")
    };

    if !dir.exists() {
        fs::create_dir_all(&dir).context(format!("Failed to create app directory: {:?}", dir))?;
    }

    Ok(dir)
}

pub fn get_origins_dir() -> Result<PathBuf> {
    let dir = get_app_dir()?.join("origins");

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .context(format!("Failed to create origins directory: {:?}", dir))?;
    }

    Ok(dir)
}

pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_app_dir()?.join("config.toml"))
}

pub fn load_config() -> Result<Config> {
    let path = get_config_path()?;

    if !path.exists() {
        let config = Config::default();
        save_config(&config)?;
        return Ok(config);
    }

    let content =
        fs::read_to_string(&path).context(format!("Failed to read config file: {:?}", path))?;

    toml::from_str(&content).context("Failed to parse config file")
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path()?;
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

    fs::write(&path, content).context(format!("Failed to write config file: {:?}", path))?;

    Ok(())
}

pub fn load_origin(id: &str) -> Result<Origin> {
    let path = get_origins_dir()?.join(format!("{}.toml", id));

    if !path.exists() {
        anyhow::bail!("Origin '{}' not found", id);
    }

    let content =
        fs::read_to_string(&path).context(format!("Failed to read origin file: {:?}", path))?;

    let mut origin: Origin = toml::from_str(&content).context("Failed to parse origin file")?;
    origin.id = id.to_string();

    Ok(origin)
}

pub fn save_origin(origin: &Origin) -> Result<()> {
    let path = get_origins_dir()?.join(format!("{}.toml", origin.id));
    let content = toml::to_string_pretty(origin).context("Failed to serialize origin")?;

    fs::write(&path, content).context(format!("Failed to write origin file: {:?}", path))?;

    Ok(())
}

pub fn delete_origin(id: &str) -> Result<()> {
    let path = get_origins_dir()?.join(format!("{}.toml", id));

    if !path.exists() {
        anyhow::bail!("Origin '{}' not found", id);
    }

    fs::remove_file(&path).context(format!("Failed to delete origin file: {:?}", path))?;

    Ok(())
}

pub fn list_origins() -> Result<Vec<String>> {
    let dir = get_origins_dir()?;
    let mut origins = Vec::new();

    for entry in fs::read_dir(&dir).context("Failed to read origins directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                origins.push(stem.to_string());
            }
        }
    }

    origins.sort();
    Ok(origins)
}

pub fn get_default_origin() -> Result<Option<String>> {
    let config = load_config()?;
    Ok(config.default_origin)
}

pub fn set_default_origin(id: &str) -> Result<()> {
    let mut config = load_config()?;
    config.default_origin = Some(id.to_string());
    save_config(&config)
}
