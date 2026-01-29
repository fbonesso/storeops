use reqwest::header::{HeaderMap, AUTHORIZATION};
use serde::de::DeserializeOwned;
use serde_json::Value;

const BASE_URL: &str = "https://api.appstoreconnect.apple.com/v1";

pub struct AppleClient {
    client: reqwest::Client,
    token: String,
}

impl AppleClient {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        h
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{BASE_URL}{path}");
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(query)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Apple API error {status}: {body}").into());
        }
        Ok(resp.json().await?)
    }

    pub async fn post(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{BASE_URL}{path}");
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Apple API error {status}: {body}").into());
        }
        Ok(resp.json().await?)
    }

    pub async fn patch(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{BASE_URL}{path}");
        let resp = self
            .client
            .patch(&url)
            .headers(self.headers())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Apple API error {status}: {body}").into());
        }
        Ok(resp.json().await?)
    }

    pub async fn delete(
        &self,
        path: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{BASE_URL}{path}");
        let resp = self
            .client
            .delete(&url)
            .headers(self.headers())
            .send()
            .await?;
        let status = resp.status();
        if status == reqwest::StatusCode::NO_CONTENT {
            return Ok(serde_json::json!({"status": "deleted"}));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Apple API error {status}: {body}").into());
        }
        Ok(resp.json().await.unwrap_or(serde_json::json!({"status": "ok"})))
    }
}
