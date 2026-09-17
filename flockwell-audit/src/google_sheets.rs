use std::error::Error;
use yup_oauth2::{AccessToken, ServiceAccountAuthenticator, ServiceAccountKey, authenticator};

const SHEETS_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/spreadsheets.readonly";

#[derive(Debug, serde::Deserialize)]
struct ValueRange {
    #[serde(default)]
    values: Vec<Vec<String>>,
}

pub(crate) async fn authenticate() -> Result<AccessToken, Box<dyn Error>> {
    let credentials_json = std::env::var("FLOCKWELL_GOOGLE_CREDENTIALS_JSON")?;
    let key: ServiceAccountKey = serde_json::from_str(&credentials_json)?;

    let authenticator = ServiceAccountAuthenticator::builder(key).build().await?;

    let token = authenticator.token(&[SHEETS_READONLY_SCOPE]).await?;

    Ok(token)
}

pub(crate) async fn fetch_animal_headers(
    token: &AccessToken,
    spreadsheet_id: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let bearer = token.token().ok_or("Missing access token")?;

    let mut url = reqwest::Url::parse("https://sheets.googleapis.com/v4/spreadsheets/")?;

    url.path_segments_mut()
        .map_err(|_| "Cannot construct Sheets URL")?
        .pop_if_empty()
        .extend(&[spreadsheet_id, "values", "'Animals'!1:1"]);

    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(bearer)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?
        .error_for_status()?
        .json::<ValueRange>()
        .await?;

    Ok(response.values.into_iter().next().unwrap_or_default())
}
