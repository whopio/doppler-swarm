mod config;
mod docker;
mod error;
mod result;
mod secrets;
mod watch;
mod worker;

#[tokio::main]
async fn main() -> crate::result::Result<()> {
    let config = config::read_config()?;

    println!("Starting {} watchers...", config.watchers.len());

    let mut handles = vec![];

    for watcher in config.watchers {
        let handle = tokio::spawn(async move {
            let fetcher = worker::Worker::new(watcher.clone());
            fetcher.run().await
        });

        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    println!("Done.");

    Ok(())
}
