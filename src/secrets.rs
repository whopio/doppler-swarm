use std::collections::HashMap;

pub async fn fetch_secrets(
    http: &reqwest::Client,
    doppler_token: &str,
) -> crate::result::Result<HashMap<String, String>> {
    let response = http
        .get("https://api.doppler.com/v3/configs/config/secrets/download?format=json")
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

    let secrets: HashMap<String, String> = response
        .json()
        .await
        .map_err(|e| format!("Cannot read response body: {}", e))?;

    Ok(secrets)
}
