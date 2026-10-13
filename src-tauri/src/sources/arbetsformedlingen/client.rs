use anyhow::{Context, Result};
use reqwest::header::ACCEPT;
use serde_json::Value;
use std::time::Duration;

pub struct AfClient {
    client: reqwest::Client,
    base_url: String,
}

impl AfClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
        })
    }

    pub async fn search_jobs(&self, query: &str, limit: u32, offset: u32, location: Option<(f64, f64, u32)>) -> Result<Value> {
        let mut parameters = vec![
            ("q", query.to_owned()),
            ("limit", limit.min(100).to_string()),
            ("offset", offset.to_string()),
        ];
        if let Some((latitude, longitude, radius_km)) = location {
            parameters.push(("position", format!("{latitude:.5},{longitude:.5}")));
            parameters.push(("position.radius", radius_km.to_string()));
        }
        self.client
            .get(format!("{}/search", self.base_url))
            .query(&parameters)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .context("Could not contact JobSearch")?
            .error_for_status()
            .context("JobSearch returned an error status")?
            .json::<Value>()
            .await
            .context("Could not parse JobSearch response")
    }
}
