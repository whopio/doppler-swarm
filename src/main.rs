mod config;
mod docker;
mod error;
mod result;
mod secrets;
mod watch;
mod worker;

#[tokio::main]
async fn main() -> crate::result::Result<()> {
    // env_logger::init();
    let env = env_logger::Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let config = config::read_config()?;

    log::info!("Starting {} watchers...", config.watchers.len());

    let mut handles = vec![];

    for watcher in config.watchers {
        let handle = tokio::spawn(async move {
            let fetcher = worker::Worker::new(watcher.clone());
            fetcher.run().await
        });

        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    log::info!("Done.");

    Ok(())
}
