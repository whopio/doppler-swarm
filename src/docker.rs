use crate::config::Watcher;
use bollard::service::{InspectServiceOptions, UpdateServiceOptions};

pub async fn get_current_env_vars(service_name: &str) -> crate::result::Result<Vec<String>> {
    let client = bollard::Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let service = client
        .inspect_service(service_name, None)
        .await
        .map_err(|e| format!("Failed to inspect service {}: {}", service_name, e))?;

    let mut env_vars = service
        .spec
        .and_then(|service_spec| service_spec.task_template)
        .and_then(|task_spec| task_spec.container_spec)
        .and_then(|container_spec| container_spec.env)
        .unwrap_or_else(Vec::new);

    env_vars.sort();

    Ok(env_vars)
}

pub async fn update_service(
    service_name: &str,
    env_vars: Vec<String>,
) -> crate::result::Result<()> {
    let docker = bollard::Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let current_service = docker
        .inspect_service(service_name, None::<InspectServiceOptions>)
        .await
        .map_err(|e| format!("Failed to inspect service {}: {}", service_name, e))?;

    let current_version = current_service
        .version
        .ok_or_else(|| format!("[{}] Cannot get docker service version", service_name))?
        .index
        .ok_or_else(|| format!("[{}] Cannot get docker service version index", service_name))?;

    let mut current_spec = current_service
        .spec
        .ok_or_else(|| format!("[{}] Cannot get docker service spec", service_name))?;

    // Update the existing ServiceSpec with new environment variables
    current_spec.name = Some(service_name.to_string());

    if let Some(task_template) = &mut current_spec.task_template {
        if let Some(container_spec) = &mut task_template.container_spec {
            container_spec.env = Some(env_vars);

            let options = UpdateServiceOptions {
                version: current_version,
                ..Default::default()
            };

            // Update the service with the modified spec
            docker
                .update_service(service_name, current_spec, options, None)
                .await
                .map_err(|e| format!("[{}] Failed to update service: {}", service_name, e))?;
        }
    }

    Ok(())
}

pub fn is_pattern(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

// Matches patters with * and ? wildcards.
pub fn is_match(string: &str, pattern: &str) -> bool {
    let m = string.len();
    let n = pattern.len();

    let mut dp = vec![vec![false; n + 1]; m + 1];

    let sb = string.as_bytes();
    let pb = pattern.as_bytes();

    dp[m][n] = true;

    for i in (0..=m).rev() {
        for j in (0..n).rev() {
            if pb[j] == b'*' {
                dp[i][j] = dp[i][j + 1];

                if i < m {
                    dp[i][j] = dp[i][j] || dp[i + 1][j] || dp[i + 1][j + 1];
                }
            } else {
                dp[i][j] = i < m && (sb[i] == pb[j] || pb[j] == b'?') && dp[i + 1][j + 1];
            }
        }
    }

    dp[0][0]
}

pub async fn list_services(watcher: &Watcher) -> crate::result::Result<Vec<String>> {
    let docker = bollard::Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let docker_services: Vec<_> = docker
        .list_services(None::<bollard::service::ListServicesOptions<String>>)
        .await
        .map_err(|e| format!("Failed to list services: {}", e))?;

    let docker_service_names = docker_services
        .iter()
        .filter_map(|service| {
            let service_name = service.spec.as_ref()?.name.clone()?;
            Some(service_name)
        })
        .collect::<Vec<_>>();

    match_services(watcher, docker_service_names).await
}

pub async fn match_services(
    watcher: &Watcher,
    docker_service_names: Vec<String>,
) -> crate::result::Result<Vec<String>> {
    let mut services = vec![];

    for service_name_pattern in &watcher.docker_services {
        if is_pattern(service_name_pattern) {
            let mut count = 0;
            for docker_service_name in &docker_service_names {
                if !docker_service_name.is_empty()
                    && is_match(&docker_service_name, service_name_pattern)
                {
                    count += 1;
                    if services.contains(docker_service_name) {
                        return Err(
                            "Configuration error: service name cannot be used multiple times"
                                .into(),
                        );
                    } else {
                        services.push(docker_service_name.to_owned());
                    }
                }

                if count == 0 {
                    return Err(format!(
                        "Configuration error: no services match pattern {}",
                        service_name_pattern
                    )
                    .into());
                }
            }
        } else {
            if !docker_service_names.contains(&service_name_pattern) {
                return Err(format!(
                    "Configuration error: service {} does not exist",
                    service_name_pattern
                )
                .into());
            }

            if services.contains(service_name_pattern) {
                return Err(
                    "Configuration error: service name cannot be used multiple times".into(),
                );
            } else {
                services.push(service_name_pattern.to_owned());
            }
        }
    }

    Ok(services)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pattern() {
        assert!(is_pattern("pattern_with_asterisk*"));
        assert!(is_pattern("pattern_with_question?"));
        assert!(is_pattern("complex*?pattern"));
        assert!(!is_pattern("no_wildcards_pattern"));
    }

    #[test]
    fn test_is_match_exact_match() {
        assert!(is_match("exact_match", "exact_match"));
    }

    #[test]
    fn test_is_match_question_start() {
        assert!(is_match("pattern_start", "??ttern_start"));
        assert!(!is_match("pattern_start", "?pattern_start"));
    }

    #[test]
    fn test_is_match_question_middle() {
        assert!(is_match("pattern_middle", "pat?ern_middle"));
        assert!(!is_match("pattern_middle", "pat?tern_middle"));
    }

    #[test]
    fn test_is_match_question_end() {
        assert!(is_match("pattern_end", "pattern_en?"));
        assert!(!is_match("pattern_end", "pattern_en?d"));
    }

    #[test]
    fn test_is_match_star_start() {
        assert!(is_match("pattern_start", "*ttern_start"));
        assert!(is_match("pattern_start", "*pattern_start"));
    }

    #[test]
    fn test_is_match_star_middle() {
        assert!(is_match("pattern_middle", "pat*ern_middle"));
        assert!(is_match("pattern_middle", "pat*tern_middle"));
    }

    #[test]
    fn test_is_match_star_end() {
        assert!(is_match("pattern_end", "pattern_e*"));
        assert!(is_match("pattern_end", "pattern_en*"));
    }

    #[tokio::test]
    async fn test_match_services_exact_match() {
        let watcher = Watcher {
            name: "My watcher".to_owned(),
            docker_services: vec!["service1".to_owned(), "service2".to_owned()],
            doppler_token: "secret".to_owned(),
        };

        let docker_service_names = vec!["service1".to_owned(), "service2".to_owned()];

        let result = match_services(&watcher, docker_service_names).await;
        assert_eq!(
            result,
            Ok(vec!["service1".to_owned(), "service2".to_owned()])
        );
    }

    #[tokio::test]
    async fn test_match_services_pattern_and_filtered() {
        let watcher = Watcher {
            name: "My watcher".to_owned(),
            docker_services: vec!["service*".to_owned()],
            doppler_token: "secret".to_owned(),
        };

        let docker_service_names = vec![
            "service1".to_owned(),
            "service2".to_owned(),
            "another_service".to_owned(),
        ];

        let result = match_services(&watcher, docker_service_names).await;
        assert_eq!(
            result,
            Ok(vec!["service1".to_owned(), "service2".to_owned()])
        );
    }

    #[tokio::test]
    async fn test_match_services_unknown_service() {
        let watcher = Watcher {
            name: "My watcher".to_owned(),
            docker_services: vec!["service1".to_owned()],
            doppler_token: "secret".to_owned(),
        };

        let docker_service_names = vec!["service2".to_owned(), "another_service".to_owned()];

        let result = match_services(&watcher, docker_service_names).await;
        assert_eq!(
            result,
            Err("Configuration error: service service1 does not exist".into())
        );
    }

    #[tokio::test]
    async fn test_match_services_unknown_service_by_pattern() {
        let watcher = Watcher {
            name: "My watcher".to_owned(),
            docker_services: vec!["service*".to_owned()],
            doppler_token: "secret".to_owned(),
        };

        let docker_service_names = vec!["another_service".to_owned()];

        let result = match_services(&watcher, docker_service_names).await;
        assert_eq!(
            result,
            Err("Configuration error: no services match pattern service*".into())
        );
    }
}
