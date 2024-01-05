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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            watchers: vec![
                Watcher {
                    name: "watcher1".to_string(),
                    doppler_token: "token1".to_string(),
                    docker_services: vec!["service1".to_string(), "service2".to_string()],
                },
                Watcher {
                    name: "watcher2".to_string(),
                    doppler_token: "token2".to_string(),
                    docker_services: vec!["service3".to_string()],
                },
            ],
        };

        let result = validate_config(&config);
        assert!(result.is_ok(), "Expected Ok result");
    }

    #[test]
    fn test_validate_config_empty_watcher_name() {
        let config = Config {
            watchers: vec![Watcher {
                name: "".to_string(),
                doppler_token: "token1".to_string(),
                docker_services: vec!["service1".to_string()],
            }],
        };

        let result = validate_config(&config);
        assert!(result.is_err(), "Expected Err result");
        assert_eq!(
            result.err().unwrap().to_string(),
            "Configuration error: watcher name cannot be empty"
        );
    }

    #[test]
    fn test_validate_config_empty_doppler_token() {
        let config = Config {
            watchers: vec![Watcher {
                name: "watcher1".to_string(),
                doppler_token: "".to_string(),
                docker_services: vec!["service1".to_string()],
            }],
        };

        let result = validate_config(&config);
        assert!(result.is_err(), "Expected Err result");
        assert_eq!(
            result.err().unwrap().to_string(),
            "Configuration error: doppler token cannot be empty"
        );
    }

    #[test]
    fn test_validate_config_empty_docker_services() {
        let config = Config {
            watchers: vec![Watcher {
                name: "watcher1".to_string(),
                doppler_token: "token1".to_string(),
                docker_services: vec![],
            }],
        };

        let result = validate_config(&config);
        assert!(result.is_err(), "Expected Err result");
        assert_eq!(
            result.err().unwrap().to_string(),
            "Configuration error: docker services cannot be empty"
        );
    }

    #[test]
    fn test_validate_config_duplicate_services() {
        let config = Config {
            watchers: vec![
                Watcher {
                    name: "watcher1".to_string(),
                    doppler_token: "token1".to_string(),
                    docker_services: vec!["service1".to_string(), "service2".to_string()],
                },
                Watcher {
                    name: "watcher2".to_string(),
                    doppler_token: "token2".to_string(),
                    docker_services: vec!["service1".to_string()],
                },
            ],
        };

        let result = validate_config(&config);
        assert!(result.is_err(), "Expected Err result");
        assert_eq!(
            result.err().unwrap().to_string(),
            "Configuration error: service service1 is used in multiple watchers"
        );
    }
}
