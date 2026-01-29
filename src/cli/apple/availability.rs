use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum AvailabilityCommand {
    /// Get app territory availability
    Get {
        /// App ID
        app_id: String,
    },
    /// List all available territories
    Territories,
    /// Set territory availability for an app
    Set {
        /// App ID
        app_id: String,
        /// Territory IDs (comma-separated, e.g., USA,GBR,JPN)
        #[arg(long, value_delimiter = ',')]
        territories: Vec<String>,
    },
}

pub async fn handle(
    cmd: &AvailabilityCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AvailabilityCommand::Get { app_id } => {
            client
                .get::<Value>(
                    &format!("/apps/{app_id}/appAvailability"),
                    &[("include", "availableTerritories")],
                )
                .await
        }
        AvailabilityCommand::Territories => {
            let limit_str = limit.unwrap_or(200).to_string();
            client
                .get("/territories", &[("limit", limit_str.as_str())])
                .await
        }
        AvailabilityCommand::Set {
            app_id,
            territories,
        } => {
            let territory_data: Vec<Value> = territories
                .iter()
                .map(|t| json!({ "type": "territories", "id": t }))
                .collect();
            let body = json!({
                "data": {
                    "type": "appAvailabilities",
                    "attributes": {
                        "availableInNewTerritories": false
                    },
                    "relationships": {
                        "app": {
                            "data": { "type": "apps", "id": app_id }
                        },
                        "availableTerritories": {
                            "data": territory_data
                        }
                    }
                }
            });
            client.post("/appAvailabilities", &body).await
        }
    }
}
