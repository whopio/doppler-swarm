use std::collections::HashMap;

use crate::config::Watcher;

pub async fn get_current_env_vars(
    service_name: &str,
) -> crate::result::Result<HashMap<String, String>> {
    let mut child = tokio::process::Command::new("docker")
        .arg("service")
        .arg("inspect")
        .arg("--format")
        .arg("{{json .Spec.TaskTemplate.ContainerSpec.Env}}")
        .arg(service_name)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker service inspect command: {e}"))?;

    log::info!("Running \"docker service inspect --format {{json .Spec.TaskTemplate.ContainerSpec.Env}} {}\"", service_name);

    let stdout = child.stdout.take().unwrap();

    let mut buf = Vec::new();

    tokio::io::copy(&mut tokio::io::BufReader::new(stdout), &mut buf)
        .await
        .map_err(|_e| {
            format!(
                "Failed to read docker service inspect output: {}",
                String::from_utf8_lossy(&buf)
            )
        })?;

    let env_var_pairs: Option<Vec<String>> = serde_json::from_slice(&buf).map_err(|_e| {
        format!(
            "Failed to parse docker service inspect output: {}",
            String::from_utf8_lossy(&buf)
        )
    })?;

    let Some(env_var_pairs) = env_var_pairs else {
        return Ok(HashMap::new());
    };

    let mut env_vars = HashMap::new();

    for pair in env_var_pairs {
        let (name, value) = parse_env_pair(&pair)?;
        env_vars.insert(name, value);
    }

    Ok(env_vars)
}

fn parse_env_pair(env_var: &str) -> crate::result::Result<(String, String)> {
    match env_var.split_once('=') {
        Some((name, value)) => {
            if name.is_empty() {
                return Err(format!("Cannot parse env var: {}", env_var).into());
            }

            if value.is_empty() {
                return Err(format!("Cannot parse env var: {}", env_var).into());
            }

            Ok((name.to_owned(), value.to_owned()))
        }
        None => Err(format!("Cannot parse env var: {env_var}").into()),
    }
}

pub fn list_env_vars_to_delete(
    old_env_vars: HashMap<String, String>,
    new_env_vars: HashMap<String, String>,
) -> crate::result::Result<Vec<String>> {
    let mut env_vars_to_delete = vec![];

    new_env_vars.keys().for_each(|new_env_var_name| {
        // check that old env vars contain the new env var name
        if !old_env_vars.contains_key(new_env_var_name) {
            env_vars_to_delete.push(new_env_var_name.to_owned());
        }
    });

    Ok(env_vars_to_delete)
}

pub fn list_env_pairs_to_update(
    old_env_vars: HashMap<String, String>,
    new_env_vars: HashMap<String, String>,
) -> crate::result::Result<HashMap<String, String>> {
    let mut env_vars_to_update = HashMap::new();

    for (new_env_var_name, new_env_var_value) in new_env_vars {
        // check that old env vars contain the new env var name
        match old_env_vars.get(&new_env_var_name) {
            None => {
                env_vars_to_update.insert(new_env_var_name, new_env_var_value);
            }
            Some(old_env_var_value) => {
                if old_env_var_value != &new_env_var_value {
                    env_vars_to_update.insert(new_env_var_name, new_env_var_value);
                }
            }
        }
    }

    Ok(env_vars_to_update)
}

pub async fn update_service(
    service_name: &str,
    old_env_vars: HashMap<String, String>,
    new_env_vars: HashMap<String, String>,
) -> crate::result::Result<()> {
    // dbg!(service_name, &old_env_vars, &new_env_vars);

    let env_vars_to_delete = list_env_vars_to_delete(old_env_vars.clone(), new_env_vars.clone())?;
    let env_vars_to_update = list_env_pairs_to_update(old_env_vars, new_env_vars)?;

    if env_vars_to_delete.is_empty() && env_vars_to_update.is_empty() {
        log::info!("No changes to apply to {}", service_name);
        return Ok(());
    }

    let mut command = tokio::process::Command::new("docker");
    command.arg("service");
    command.arg("update");

    let mut args_info = String::new();

    for env_var in env_vars_to_delete {
        command.arg("--env-rm").arg(&env_var);
        args_info.push_str(&format!("--env-rm {} ", env_var));
    }

    for (env_var_name, env_var_value) in env_vars_to_update {
        let arg = format!("{}={}", env_var_name, env_var_value);
        command.arg("--env-add").arg(&arg);
        // dbg!(&env_var);
        // println!("env_var: {}", &env_var);
        args_info.push_str(&format!("--env-add \"{}\" ", arg));
    }

    args_info.pop();

    let mut child = command
        .arg(service_name)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker service inspect command: {e}"))?;

    log::info!(
        "Running \"docker service update {} {}\"",
        args_info,
        service_name
    );

    let stdout = child.stdout.take().unwrap();

    let mut buf = Vec::new();

    tokio::io::copy(&mut tokio::io::BufReader::new(stdout), &mut buf)
        .await
        .map_err(|_e| {
            format!(
                "Failed to read docker service inspect output: {}",
                String::from_utf8_lossy(&buf)
            )
        })?;

    Ok(())
}

pub fn is_pattern(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

// Matches patters with * and ? wildcards.
pub fn is_match(text: &str, pattern: &str) -> bool {
    let m = text.len();
    let n = pattern.len();

    let mut dp = vec![vec![false; n + 1]; m + 1];

    let text_bytes = text.as_bytes();
    let pattern_bytes = pattern.as_bytes();

    dp[0][0] = true;

    for j in 1..=n {
        if pattern_bytes[j - 1] == b'*' {
            dp[0][j] = dp[0][j - 1];
        }
    }

    for i in 1..=m {
        for j in 1..=n {
            if pattern_bytes[j - 1] == b'*' {
                dp[i][j] = dp[i][j - 1] || dp[i - 1][j];
            } else if pattern_bytes[j - 1] == b'?' || text_bytes[i - 1] == pattern_bytes[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }

    dp[m][n]
}

pub async fn list_services(watcher: &Watcher) -> crate::result::Result<Vec<String>> {
    let mut child = tokio::process::Command::new("docker")
        .arg("service")
        .arg("ls")
        .arg("--format")
        .arg("{{.Name}}")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker service ls command: {e}"))?;

    log::info!("Running \"docker service ls --format {{.Name}}\"");

    let stdout = child.stdout.take().unwrap();

    let mut buf = Vec::new();

    tokio::io::copy(&mut tokio::io::BufReader::new(stdout), &mut buf)
        .await
        .map_err(|_e| {
            format!(
                "Failed to read docker service ls output: {}",
                String::from_utf8_lossy(&buf)
            )
        })?;

    let docker_service_names: Vec<String> = String::from_utf8_lossy(&buf)
        .split('\n')
        .filter(|service_name| !service_name.is_empty())
        .map(|service_name| service_name.to_owned())
        .collect();

    log::info!(
        "[{}] Found {} docker services: {:?}",
        &watcher.name,
        docker_service_names.len(),
        &docker_service_names
    );

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
                    && is_match(docker_service_name, service_name_pattern)
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
            }

            if count == 0 {
                return Err(format!(
                    "Configuration error: no services match pattern {}",
                    service_name_pattern
                )
                .into());
            }
        } else {
            if !docker_service_names.contains(service_name_pattern) {
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
    fn test_list_env_pairs_to_update_no_changes() {
        let mut old_env_vars = HashMap::new();
        old_env_vars.insert("VAR1".to_string(), "old_value1".to_string());
        old_env_vars.insert("VAR2".to_string(), "old_value2".to_string());

        let mut new_env_vars = HashMap::new();
        new_env_vars.insert("VAR1".to_string(), "old_value1".to_string());
        new_env_vars.insert("VAR2".to_string(), "old_value2".to_string());

        let result = list_env_pairs_to_update(old_env_vars, new_env_vars).unwrap();
        assert!(result.is_empty()); // No changes, so the result should be an empty HashMap
    }

    #[test]
    fn test_list_env_pairs_to_update_with_changes() {
        let mut old_env_vars = HashMap::new();
        old_env_vars.insert("VAR1".to_string(), "old_value1".to_string());
        old_env_vars.insert("VAR2".to_string(), "old_value2".to_string());

        let mut new_env_vars = HashMap::new();
        new_env_vars.insert("VAR1".to_string(), "new_value1".to_string());
        new_env_vars.insert("VAR2".to_string(), "old_value2".to_string());

        let result = list_env_pairs_to_update(old_env_vars, new_env_vars).unwrap();
        let mut expected_result = HashMap::new();
        expected_result.insert("VAR1".to_string(), "new_value1".to_string());
        assert_eq!(result, expected_result); // VAR1 has changed, so it should be in the result
    }

    #[test]
    fn test_list_env_pairs_to_update_missing_old_vars() {
        let mut old_env_vars = HashMap::new();
        old_env_vars.insert("VAR1".to_string(), "old_value1".to_string());

        let mut new_env_vars = HashMap::new();
        new_env_vars.insert("VAR1".to_string(), "new_value1".to_string());
        new_env_vars.insert("VAR2".to_string(), "new_value2".to_string());

        let result = list_env_pairs_to_update(old_env_vars, new_env_vars).unwrap();
        let mut expected_result = HashMap::new();
        expected_result.insert("VAR1".to_string(), "new_value1".to_string());
        expected_result.insert("VAR2".to_string(), "new_value2".to_string());
        // VAR1 has changed, VAR2 is a new variable, both should be in the result
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_list_env_pairs_to_update_new_vars_not_in_old_vars() {
        let mut old_env_vars = HashMap::new();
        old_env_vars.insert("VAR1".to_string(), "old_value1".to_string());

        let mut new_env_vars = HashMap::new();
        new_env_vars.insert("VAR2".to_string(), "new_value2".to_string());
        new_env_vars.insert("VAR3".to_string(), "new_value3".to_string());

        let result = list_env_pairs_to_update(old_env_vars, new_env_vars).unwrap();
        let mut expected_result = HashMap::new();
        expected_result.insert("VAR2".to_string(), "new_value2".to_string());
        expected_result.insert("VAR3".to_string(), "new_value3".to_string());
        // VAR2 and VAR3 are new variables, both should be in the result
        assert_eq!(result, expected_result);
    }

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
        assert!(!is_match("exact_match", "exact_matchz"));
    }

    #[test]
    fn test_is_match_question_start() {
        assert!(is_match("pattern_start", "??ttern_start"));
        assert!(is_match("pattern_start", "?attern_start"));
    }

    #[test]
    fn test_is_match_question_middle() {
        assert!(is_match("pattern_middle", "pat?ern_middle"));
        assert!(is_match("pattern_middle", "pat?ern_?iddle"));
        assert!(is_match("pattern_middle", "pat?e??_?iddle"));
        assert!(is_match("pattern_middle", "pat??????iddle"));
    }

    #[test]
    fn test_is_match_question_end() {
        assert!(is_match("pattern_end", "pattern_en?"));
        assert!(is_match("pattern_end", "pattern_e??"));
    }

    #[test]
    fn test_is_match_star_start() {
        assert!(is_match("pattern_start", "*ttern_start"));
        assert!(is_match("pattern_start", "*pattern_start"));
        assert!(is_match("pattern_start", "*a*t?r*_*t?r?*"));
        assert!(is_match("main_app", "*_app"));
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

    #[tokio::test]
    async fn test_match_services_unknown_match() {
        let watcher = Watcher {
            name: "My watcher".to_owned(),
            docker_services: vec!["my*".to_owned()],
            doppler_token: "secret".to_owned(),
        };

        let docker_service_names = vec!["myservice1".to_owned(), "myservice2".to_owned()];

        let result = match_services(&watcher, docker_service_names).await;
        assert_eq!(
            result,
            Ok(vec!["myservice1".to_owned(), "myservice2".to_owned()])
        );
    }
}
