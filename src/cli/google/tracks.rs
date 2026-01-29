use clap::Subcommand;
use serde_json::Value;

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum TracksCommand {
    /// List all tracks
    List {
        /// Package name
        package_name: String,
    },
    /// Update a track release
    Update {
        /// Package name
        package_name: String,
        /// Track name (internal, alpha, beta, production)
        #[arg(long)]
        track: String,
        /// Release JSON body
        #[arg(long)]
        release: String,
    },
}

pub async fn handle(
    cmd: &TracksCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        TracksCommand::List { package_name } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &serde_json::json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let tracks: Value = client
                .get(&format!("/{package_name}/edits/{edit_id}/tracks"), &[])
                .await?;
            let _ = client
                .delete_path(&format!("/{package_name}/edits/{edit_id}"))
                .await;
            Ok(tracks)
        }
        TracksCommand::Update {
            package_name,
            track,
            release,
        } => {
            let release_json: Value = serde_json::from_str(release)?;
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &serde_json::json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result = client
                .put(
                    &format!("/{package_name}/edits/{edit_id}/tracks/{track}"),
                    &serde_json::json!({
                        "track": track,
                        "releases": [release_json]
                    }),
                )
                .await?;
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
