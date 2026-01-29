use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

pub async fn handle(
    app_id: &str,
    version: &str,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    let versions: Value = client
        .get(
            &format!("/apps/{app_id}/appStoreVersions"),
            &[("filter[versionString]", version)],
        )
        .await?;

    let version_id = versions["data"][0]["id"]
        .as_str()
        .ok_or("version not found")?;

    let body = json!({
        "data": {
            "type": "appStoreVersionSubmissions",
            "relationships": {
                "appStoreVersion": {
                    "data": {
                        "type": "appStoreVersions",
                        "id": version_id
                    }
                }
            }
        }
    });

    client.post("/appStoreVersionSubmissions", &body).await
}
