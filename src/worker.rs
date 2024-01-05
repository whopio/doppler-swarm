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
        log::info!("Fetching secrets for {}", &self.watcher.name);

        self.sync_secrets().await;
        self.watch_for_updates().await;
    }

    pub async fn sync_secrets(&self) {
        let doppler_secrets = fetch_secrets(&self.http, &self.watcher.doppler_token)
            .await
            .unwrap();

        for service in &self.watcher.docker_services {
            let docker_secrets = crate::docker::get_current_env_vars(service).await.unwrap();

            if should_update_docker_service(&doppler_secrets, &docker_secrets) {
                crate::docker::update_service(service, doppler_secrets.clone())
                    .await
                    .unwrap();

                log::info!("Updated {}", service);
            } else {
                log::info!("No changes detected for {}", service);
            }
        }
    }

    pub async fn watch_for_updates(&self) {
        let response = self
            .http
            .get("https://api.doppler.com/v3/configs/config/secrets/watch?include_dynamic_secrets=false&include_managed_secrets=false")
            .bearer_auth(&self.watcher.doppler_token)
            .send()
            .await
            .expect("request failed");

        let mut stream = response.bytes_stream();
        while let Some(Ok(item)) = stream.next().await {
            let Ok(event) = parse_watch_event(&item) else {
                continue;
            };

            if WatchEvent::SecretsUpdate == event {
                self.sync_secrets().await;
            }
        }
    }
}
