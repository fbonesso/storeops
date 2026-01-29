use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum AvailabilityCommand {
    /// Get country targeting / availability for a track
    Get {
        /// Package name
        package_name: String,
        /// Track name (e.g., production, beta)
        #[arg(long)]
        track: String,
    },
    /// List all available country codes
    Countries {
        /// Package name
        package_name: String,
    },
    /// Update country targeting for a release
    Update {
        /// Package name
        package_name: String,
        /// Track name
        #[arg(long)]
        track: String,
        /// Country codes to include (comma-separated, e.g., US,GB,JP)
        #[arg(long, value_delimiter = ',')]
        countries: Vec<String>,
        /// Whether to target rest of world by default
        #[arg(long)]
        rest_of_world: Option<bool>,
    },
}

pub async fn handle(
    cmd: &AvailabilityCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AvailabilityCommand::Get {
            package_name,
            track,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result: Value = client
                .get(
                    &format!("/{package_name}/edits/{edit_id}/tracks/{track}/countryAvailability"),
                    &[],
                )
                .await?;
            let _ = client
                .delete_path(&format!("/{package_name}/edits/{edit_id}"))
                .await;
            Ok(result)
        }
        AvailabilityCommand::Countries { package_name } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result: Value = client
                .get(
                    &format!("/{package_name}/edits/{edit_id}/countryAvailability"),
                    &[],
                )
                .await?;
            let _ = client
                .delete_path(&format!("/{package_name}/edits/{edit_id}"))
                .await;
            Ok(result)
        }
        AvailabilityCommand::Update {
            package_name,
            track,
            countries,
            rest_of_world,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;

            let country_targeting: Vec<Value> = countries
                .iter()
                .map(|c| json!({ "countryCode": c }))
                .collect();

            let body = json!({
                "countries": country_targeting,
                "restOfWorld": rest_of_world.unwrap_or(false)
            });

            let result = client
                .put(
                    &format!("/{package_name}/edits/{edit_id}/tracks/{track}/countryAvailability"),
                    &body,
                )
                .await?;
            client
                .post(
                    &format!("/{package_name}/edits/{edit_id}:commit"),
                    &json!({}),
                )
                .await?;
            Ok(result)
        }
    }
}
