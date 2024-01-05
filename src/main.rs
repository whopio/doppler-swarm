mod config;
mod docker;
mod error;
mod result;
mod secrets;
mod watch;
mod worker;

#[tokio::main]
async fn main() -> crate::result::Result<()> {
    let env = env_logger::Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config = config::read_config()?;

    log::info!("Starting {} watchers...", config.watchers.len());

    let mut handles = vec![];

    {
        let mut fetchers = Vec::with_capacity(config.watchers.len());

        for watcher in config.watchers {
            let fetcher = worker::Worker::new(watcher.clone());
            if let Err(e) = fetcher.sync_secrets().await {
                return Err(error::Error::from(format!(
                    "[{}] Failed to sync secrets: {}",
                    &watcher.name, e
                )));
            }

            fetchers.push(fetcher);
        }

        for fetcher in fetchers {
            let handle = tokio::spawn(async move {
                fetcher.run().await;
            });

            handles.push(handle);
        }
    }

    futures::future::join_all(handles).await;

    log::info!("Done.");

    Ok(())
}
