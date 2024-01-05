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

pub fn read_config() -> crate::result::Result<Config> {
    let config_file = std::env::args().nth(1).ok_or("no config file specified")?;

    let data = std::fs::read_to_string(&config_file)
        .map_err(|e| format!("Failed to read config file {}: {}", &config_file, e))?;

    let config = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse config file {}: {}", &config_file, e))?;

    validate_config(&config)?;

    Ok(config)
}

pub fn validate_config(config: &Config) -> crate::result::Result<()> {
    let mut services_seen = vec![];
    let mut services_seen_twice = vec![];
    for watcher in &config.watchers {
        if watcher.name.is_empty() {
            return Err("Configuration error: watcher name cannot be empty".into());
        }

        if watcher.doppler_token.is_empty() {
            return Err("Configuration error: doppler token cannot be empty".into());
        }

        if watcher.docker_services.is_empty() {
            return Err("Configuration error: docker services cannot be empty".into());
        }

        for service in &watcher.docker_services {
            if services_seen.contains(service) {
                services_seen_twice.push(service.to_owned());
            } else {
                services_seen.push(service.to_owned());
            }
        }
    }

    for service in services_seen_twice {
        return Err(format!(
            "Configuration error: service {} is used in multiple watchers",
            service
        )
        .into());
    }

    Ok(())
}
