pub async fn fetch_secrets(
    http: &reqwest::Client,
    doppler_token: &str,
) -> crate::result::Result<Vec<String>> {
    let response = http
        .get("https://api.doppler.com/v3/configs/config/secrets/download?format=docker")
        .bearer_auth(doppler_token)
        .send()
        .await
        .map_err(|e| format!("{e}"))?;

    match response.status() {
        reqwest::StatusCode::OK => {}
        reqwest::StatusCode::UNAUTHORIZED => {
            return Err("INVALID DOPPLER TOKEN".into());
        }
        _ => return Err(format!("HTTP Status {}", response.status()).into()),
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Cannot read response body: {}", e))?;

    let mut secrets: Vec<String> = body.lines().map(|line| line.to_owned()).collect();

    secrets.sort();

    Ok(secrets)
}
