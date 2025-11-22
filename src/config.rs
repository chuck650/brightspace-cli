use anyhow::Result;
use config::{Config as AppConfig, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
    pub text2qti_path: String,
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("brightspace");
    Ok(config_dir.join("brightspace-cli.yaml"))
}

impl Config {
    pub fn new() -> Self {
        Self {
            base_url: "https://brightspace.example.com".to_string(),
            client_id: "".to_string(),
            client_secret: "".to_string(),
            username: "".to_string(),
            redirect_uri: "http://localhost:8080".to_string(),
            auth_url: "https://auth.brightspace.com/oauth2/auth".to_string(),
            token_url: "https://auth.brightspace.com/oauth2/token".to_string(),
            text2qti_path: "".to_string(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_file = get_config_path()?;

        let config = AppConfig::builder()
            .add_source(File::new("brightspace-cli.yaml", FileFormat::Yaml).required(false))
            .add_source(File::from(config_file).required(false))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    pub fn get(key: &str) -> Result<String> {
        let config = Self::load()?;
        match key {
            "base_url" => Ok(config.base_url),
            "client_id" => Ok(config.client_id),
            "client_secret" => Ok(config.client_secret),
            "username" => Ok(config.username),
            "redirect_uri" => Ok(config.redirect_uri),
            "auth_url" => Ok(config.auth_url),
            "token_url" => Ok(config.token_url),
            "text2qti_path" => Ok(config.text2qti_path),
            _ => Err(anyhow::anyhow!("Invalid config key")),
        }
    }

    pub fn set(key: &str, value: &str) -> Result<()> {
        let config_path = get_config_path()?;
        let file = fs::File::open(&config_path).ok();

        let mut yaml_value: serde_yaml::Value = if let Some(file) = file {
            serde_yaml::from_reader(file)?
        } else {
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
        };

        if let serde_yaml::Value::Mapping(mapping) = &mut yaml_value {
            mapping.insert(
                serde_yaml::Value::String(key.to_string()),
                serde_yaml::Value::String(value.to_string()),
            );
        }

        let yaml = serde_yaml::to_string(&yaml_value)?;
        fs::write(config_path, yaml)?;

        Ok(())
    }
}
