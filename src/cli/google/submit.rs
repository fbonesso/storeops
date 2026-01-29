use serde_json::{json, Value};

use crate::api::google_client::GoogleClient;

pub async fn handle(
    package_name: &str,
    track: &str,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    let edit: Value = client
        .post(&format!("/{package_name}/edits"), &json!({}))
        .await?;
    let edit_id = edit["id"].as_str().ok_or("no edit id")?;

    let track_info: Value = client
        .get(
            &format!("/{package_name}/edits/{edit_id}/tracks/{track}"),
            &[],
        )
        .await?;

    let commit: Value = client
        .post(
            &format!("/{package_name}/edits/{edit_id}:commit"),
            &json!({}),
        )
        .await?;

    Ok(json!({
        "track": track_info,
        "commit": commit,
    }))
}
