use clap::Subcommand;
use serde_json::Value;

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum BuildsCommand {
    /// List bundles/APKs
    List {
        /// Package name
        package_name: String,
    },
    /// Upload a bundle/APK
    Upload {
        /// Package name
        package_name: String,
        /// Path to .aab or .apk file
        #[arg(long)]
        file: String,
    },
}

pub async fn handle(
    cmd: &BuildsCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        BuildsCommand::List { package_name } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &serde_json::json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let bundles: Value = client
                .get(
                    &format!("/{package_name}/edits/{edit_id}/bundles"),
                    &[],
                )
                .await?;
            let _ = client
                .get::<Value>(&format!("/{package_name}/edits/{edit_id}:delete"), &[])
                .await;
            Ok(bundles)
        }
        BuildsCommand::Upload {
            package_name,
            file,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &serde_json::json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result = client.upload_file(package_name, edit_id, file).await?;
            let _ = client
                .post(
                    &format!("/{package_name}/edits/{edit_id}:commit"),
                    &serde_json::json!({}),
                )
                .await?;
            Ok(result)
        }
    }
}
