use crate::worker::Worker;
use tokio::task::JoinError;

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

    let mut handles = Vec::with_capacity(config.watchers.len());

    {
        let mut startup_handles = Vec::with_capacity(config.watchers.len());

        for watcher in config.watchers {
            let startup_handle = tokio::spawn(async move {
                let fetcher = worker::Worker::new(watcher.clone());
                if let Err(e) = fetcher.sync_secrets().await {
                    let error_msg = format!("[{}] Failed to sync secrets: {}", &watcher.name, e);
                    log::error!("{error_msg}");
                    return Err(error_msg.into());
                }

                Ok(fetcher)
            });

            startup_handles.push(startup_handle);
        }

        let fetchers: Vec<Result<crate::result::Result<Worker>, JoinError>> =
            futures::future::join_all(startup_handles).await;

        for fetcher_result in fetchers {
            match fetcher_result {
                Ok(Ok(fetcher)) => {
                    let handle = tokio::spawn(async move {
                        fetcher.run().await;
                    });

                    handles.push(handle);
                }
                Ok(Err(e)) => {
                    log::error!("Failed to start watcher: {e}");
                    return Err(e);
                }
                _ => {
                    let error_msg = format!("Failed to start watcher: {:?}", fetcher_result);
                    log::error!("{error_msg}");
                    return Err(error_msg.into());
                }
            }
        }
    }

    futures::future::join_all(handles).await;

    log::info!("Done.");

    Ok(())
}
