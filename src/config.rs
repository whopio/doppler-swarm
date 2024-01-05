use crate::error::Error;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Watcher {
    pub name: String,
    pub doppler_token: String,
    pub docker_services: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub watchers: Vec<Watcher>,
}

pub fn read_config() -> Result<Config, Error> {
    let config_file = std::env::args().nth(1).ok_or("no config file specified")?;

    let data = std::fs::read_to_string(&config_file)
        .map_err(|e| format!("Failed to read config file {}: {}", &config_file, e))?;

    let config = serde_json::from_str(&data)
        .map_err(Error::from)
        .map_err(|e| format!("Failed to parse config file {}: {}", &config_file, e))?;

    Ok(config)
}
