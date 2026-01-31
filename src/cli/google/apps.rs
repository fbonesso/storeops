use clap::Subcommand;
use serde_json::Value;

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum AppsCommand {
    /// Get app details from Google Play Console
    ///
    /// Examples:
    ///   storeops google apps info com.example.app
    Info {
        /// Your app's package name (e.g., com.example.app)
        package_name: String,
    },
}

pub async fn handle(
    cmd: &AppsCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AppsCommand::Info { package_name } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &serde_json::json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let details: Value = client
                .get(&format!("/{package_name}/edits/{edit_id}/details"), &[])
                .await?;
            let _ = client
                .delete_path(&format!("/{package_name}/edits/{edit_id}"))
                .await;
            Ok(details)
        }
    }
}
