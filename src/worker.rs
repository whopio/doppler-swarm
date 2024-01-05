use futures::StreamExt;

use crate::{
    config,
    secrets::fetch_secrets,
    watch::{parse_watch_event, WatchEvent},
};

pub struct Worker {
    watcher: config::Watcher,
    http: reqwest::Client,
}

pub fn should_update_docker_service(
    doppler_secrets: &Vec<String>,
    docker_secrets: &Vec<String>,
) -> bool {
    if doppler_secrets.len() != docker_secrets.len() {
        return true;
    }

    if doppler_secrets != docker_secrets {
        return true;
    }

    false
}

impl Worker {
    pub fn new(watcher: config::Watcher) -> Self {
        let http = reqwest::Client::new();
        Self { watcher, http }
    }

    pub async fn run(&self) {
        loop {
            if let Err(e) = self.watch_for_updates().await {
                log::error!(
                    "[{}] Failed to watch for updates: {}",
                    &self.watcher.name,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }

    pub async fn sync_secrets(&self) -> crate::result::Result<()> {
        let doppler_secrets = fetch_secrets(&self.http, &self.watcher.doppler_token)
            .await
            .map_err(|e| format!("Failed to fetch secrets: {}", e))?;

        for service in &self.watcher.docker_services {
            let docker_secrets = crate::docker::get_current_env_vars(service)
                .await
                .map_err(|e| format!("[{}] Failed to get current env vars: {}", service, e))?;

            if should_update_docker_service(&doppler_secrets, &docker_secrets) {
                crate::docker::update_service(service, doppler_secrets.clone())
                    .await
                    .map_err(|e| format!("[{}] Failed to update docker service: {}", service, e))?;

                log::info!("[{}] [{}] Service updated", &self.watcher.name, service);
            } else {
                log::info!("[{}] [{}] No changes detected", &self.watcher.name, service);
            }
        }

        Ok(())
    }

    pub async fn watch_for_updates(&self) -> crate::result::Result<()> {
        let response = self
            .http
            .get("https://api.doppler.com/v3/configs/config/secrets/watch?include_dynamic_secrets=false&include_managed_secrets=false")
            .bearer_auth(&self.watcher.doppler_token)
            .send()
            .await
            .map_err(|e| format!("[{}] Failed to watch for updates: {}", &self.watcher.name, e))?;

        let mut stream = response.bytes_stream();
        while let Some(Ok(item)) = stream.next().await {
            let Ok(event) = parse_watch_event(&item) else {
                continue;
            };

            if WatchEvent::SecretsUpdate == event {
                self.sync_secrets().await?;
            }
        }

        Ok(())
    }
}
