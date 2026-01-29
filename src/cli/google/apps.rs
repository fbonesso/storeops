use clap::Subcommand;
use serde_json::Value;

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum AppsCommand {
    /// List apps (requires listing from edits or known packages)
    List,
    /// Get app details
    Info {
        /// Package name (e.g., com.example.app)
        package_name: String,
    },
}

pub async fn handle(
    cmd: &AppsCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AppsCommand::List => {
            // Google Play Developer API doesn't have a direct "list apps" endpoint.
            // Return a helpful message.
            Ok(serde_json::json!({
                "error": "Google Play API does not provide a list-apps endpoint. Use 'apps info <package-name>' with a known package name."
            }))
        }
        AppsCommand::Info { package_name } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &serde_json::json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let details: Value = client
                .get(&format!("/{package_name}/edits/{edit_id}/details"), &[])
                .await?;
            let _ = client
                .get::<Value>(&format!("/{package_name}/edits/{edit_id}:delete"), &[])
                .await;
            Ok(details)
        }
    }
}
