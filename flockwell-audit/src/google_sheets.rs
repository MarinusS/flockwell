use std::error::Error;
use yup_oauth2::{AccessToken, ServiceAccountAuthenticator, ServiceAccountKey};

const SHEETS_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/spreadsheets.readonly";
const SHEET_ID: &str = "1i3cuiIWHw4vefJTLJ5Gq-3opdnsEXuu4J7pavTfBX3M";

const ANIMALS_SHEET_NAME: &str = "Animals";

#[derive(Debug, serde::Deserialize)]
struct ValueRange {
    #[serde(default)]
    values: Vec<Vec<String>>,
}

async fn authenticate() -> Result<AccessToken, Box<dyn Error>> {
    let credentials_json = std::env::var("FLOCKWELL_GOOGLE_CREDENTIALS_JSON")?;
    let key: ServiceAccountKey = serde_json::from_str(&credentials_json)?;

    let authenticator = ServiceAccountAuthenticator::builder(key).build().await?;

    let token = authenticator.token(&[SHEETS_READONLY_SCOPE]).await?;

    Ok(token)
}

pub struct Client {
    http: reqwest::Client,
    token: AccessToken,
}

impl Client {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let token = authenticate().await?;

        Ok(Self {
            http: reqwest::Client::new(),
            token,
        })
    }

    fn bearer_token(&self) -> Result<&str, Box<dyn Error>> {
        self.token
            .token()
            .ok_or_else(|| "Missing access token".into())
    }

    pub async fn fetch_animal_headers(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let bearer = self.bearer_token()?;

        let mut url = reqwest::Url::parse("https://sheets.googleapis.com/v4/spreadsheets/")?;

        url.path_segments_mut()
            .map_err(|_| "Cannot construct Sheets URL")?
            .pop_if_empty()
            .extend(&[SHEET_ID, "values", &format!("'{ANIMALS_SHEET_NAME}'!1:1")]);

        let response = self
            .http
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

    pub async fn fetch_animal_table(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let bearer = self.bearer_token()?;

        let mut url = reqwest::Url::parse("https://sheets.googleapis.com/v4/spreadsheets/")?;

        url.path_segments_mut()
            .map_err(|_| "Cannot construct Sheets URL")?
            .pop_if_empty()
            .extend(&[SHEET_ID, "values", ANIMALS_SHEET_NAME]);

        let response = self
            .http
            .get(url)
            .bearer_auth(bearer)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .json::<ValueRange>()
            .await?;

        Ok(response.values)
    }
}
