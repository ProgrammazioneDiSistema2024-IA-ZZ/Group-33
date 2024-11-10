use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct BackupConfig {
    pub source_directory: String,
    pub destination_directory: String,
    pub file_types: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CpuLoggingConfig {
    pub interval_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backup: BackupConfig,
    pub cpu_logging: CpuLoggingConfig,
}

pub fn read_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    // Leggi il file TOML come stringa
    let config_content = fs::read_to_string(file_path)?;

    // Deserializza il contenuto del file TOML in una struttura Config
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}

