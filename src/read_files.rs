use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct BackupConfig {
    pub source_directory: String,
    pub destination_directory: String,
    pub file_types: Vec<String>,
}

// Implementa il trait Default per BackupConfig
impl Default for BackupConfig {
    fn default() -> Self {
        BackupConfig {
            source_directory: String::from("C:\\Default\\Source"), // esempio di valore predefinito
            destination_directory: String::from("C:\\Default\\Destination"), // esempio di valore predefinito
            file_types: vec!["txt".to_string(), "jpg".to_string()], // esempio di tipi di file predefiniti
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CpuLoggingConfig {
    pub log_path: String,
}

#[derive(Debug, Deserialize, Clone)]
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

