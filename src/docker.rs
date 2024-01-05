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
        .await?;

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
                .await?;
        }
    }

    Ok(())
}
