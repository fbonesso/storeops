use reqwest::header::{HeaderMap, AUTHORIZATION};
use serde::de::DeserializeOwned;
use serde_json::Value;

const BASE_URL: &str = "https://androidpublisher.googleapis.com/androidpublisher/v3/applications";
const MAX_ERROR_LEN: usize = 512;

fn truncate_error(body: &str) -> &str {
    if body.len() <= MAX_ERROR_LEN {
        body
    } else {
        &body[..MAX_ERROR_LEN]
    }
}

pub struct GoogleClient {
    client: reqwest::Client,
    token: String,
}

impl GoogleClient {
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
            return Err(format!("Google API error {status}: {}", truncate_error(&body)).into());
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
            return Err(format!("Google API error {status}: {}", truncate_error(&body)).into());
        }
        Ok(resp.json().await?)
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{BASE_URL}{path}");
        let resp = self
            .client
            .put(&url)
            .headers(self.headers())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Google API error {status}: {}", truncate_error(&body)).into());
        }
        Ok(resp.json().await?)
    }

    pub async fn delete_path(&self, path: &str) -> Result<Value, Box<dyn std::error::Error>> {
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
            return Err(format!("Google API error {status}: {}", truncate_error(&body)).into());
        }
        Ok(resp
            .json()
            .await
            .unwrap_or(serde_json::json!({"status": "ok"})))
    }

    pub async fn upload_image(
        &self,
        package_name: &str,
        edit_id: &str,
        locale: &str,
        image_type: &str,
        file_path: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!(
            "https://androidpublisher.googleapis.com/upload/androidpublisher/v3/applications/{package_name}/edits/{edit_id}/listings/{locale}/{image_type}"
        );
        let file_bytes = tokio::fs::read(file_path).await?;
        let content_type = if file_path.ends_with(".png") {
            "image/png"
        } else {
            "image/jpeg"
        };
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .header("Content-Type", content_type)
            .body(file_bytes)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Google API upload error {status}: {}",
                truncate_error(&body)
            )
            .into());
        }
        Ok(resp.json().await?)
    }

    pub async fn upload_file(
        &self,
        package_name: &str,
        edit_id: &str,
        file_path: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!(
            "https://androidpublisher.googleapis.com/upload/androidpublisher/v3/applications/{package_name}/edits/{edit_id}/bundles"
        );
        let file_bytes = tokio::fs::read(file_path).await?;
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .header("Content-Type", "application/octet-stream")
            .body(file_bytes)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Google API upload error {status}: {}",
                truncate_error(&body)
            )
            .into());
        }
        Ok(resp.json().await?)
    }
}
