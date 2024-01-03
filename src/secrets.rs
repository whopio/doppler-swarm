use crate::error::Error;

pub async fn fetch_secrets(http: &reqwest::Client, token: &str) -> Result<Vec<String>, Error> {
    let response = http
        .get("https://api.doppler.com/v3/configs/config/secrets/download?format=docker")
        .bearer_auth(token)
        .send()
        .await
        .expect("request failed");

    if response.status() != 200 {
        return Err(format!("failed to fetch secrets: {}", response.status()).into());
    }

    let body = response.text().await.expect("failed to get response body");
    let mut secrets: Vec<String> = body.lines().map(|line| line.to_owned()).collect();

    secrets.sort();

    Ok(secrets)
}
