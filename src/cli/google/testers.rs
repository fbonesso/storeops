use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum TestersCommand {
    /// List testers for a track
    List {
        /// Package name
        package_name: String,
        /// Track name
        #[arg(long)]
        track: String,
    },
    /// Add tester to a track
    Add {
        /// Package name
        package_name: String,
        /// Track name
        #[arg(long)]
        track: String,
        /// Tester email
        #[arg(long)]
        email: String,
    },
}

pub async fn handle(
    cmd: &TestersCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        TestersCommand::List {
            package_name,
            track,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let testers: Value = client
                .get(
                    &format!("/{package_name}/edits/{edit_id}/testers/{track}"),
                    &[],
                )
                .await?;
            let _ = client
                .get::<Value>(&format!("/{package_name}/edits/{edit_id}:delete"), &[])
                .await;
            Ok(testers)
        }
        TestersCommand::Add {
            package_name,
            track,
            email,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let current: Value = client
                .get(
                    &format!("/{package_name}/edits/{edit_id}/testers/{track}"),
                    &[],
                )
                .await
                .unwrap_or(json!({"googleGroups": [], "googlePlusCommunities": []}));

            let mut emails: Vec<String> = current["googleGroups"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            emails.push(email.clone());

            let result = client
                .put(
                    &format!("/{package_name}/edits/{edit_id}/testers/{track}"),
                    &json!({ "googleGroups": emails }),
                )
                .await?;
            let _ = client
                .post(
                    &format!("/{package_name}/edits/{edit_id}:commit"),
                    &json!({}),
                )
                .await?;
            Ok(result)
        }
    }
}
