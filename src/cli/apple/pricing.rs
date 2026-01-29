use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum PricingCommand {
    /// Get current price schedule for an app
    Get {
        /// App ID
        app_id: String,
    },
    /// List available price points for an app
    Points {
        /// App ID
        app_id: String,
        /// Filter by territory (e.g., USA, GBR)
        #[arg(long)]
        territory: Option<String>,
    },
    /// Set the base price for an app
    Set {
        /// App ID
        app_id: String,
        /// Price point ID
        #[arg(long)]
        price_point: String,
        /// Start date (ISO 8601, or omit for immediate)
        #[arg(long)]
        start_date: Option<String>,
    },
}

pub async fn handle(
    cmd: &PricingCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        PricingCommand::Get { app_id } => {
            client
                .get::<Value>(
                    &format!("/apps/{app_id}/appPriceSchedule"),
                    &[("include", "manualPrices,automaticPrices")],
                )
                .await
        }
        PricingCommand::Points { app_id, territory } => {
            let limit_str = limit.unwrap_or(50).to_string();
            let mut query = vec![("limit", limit_str.as_str())];
            let territory_val;
            if let Some(t) = territory {
                territory_val = t.clone();
                query.push(("filter[territory]", &territory_val));
            }
            client
                .get(&format!("/apps/{app_id}/appPricePoints"), &query)
                .await
        }
        PricingCommand::Set {
            app_id,
            price_point,
            start_date,
        } => {
            let mut attrs = json!({});
            if let Some(sd) = start_date {
                attrs["startDate"] = json!(sd);
            }
            let body = json!({
                "data": {
                    "type": "appPriceSchedules",
                    "relationships": {
                        "app": {
                            "data": { "type": "apps", "id": app_id }
                        },
                        "manualPrices": {
                            "data": [{
                                "type": "appPrices",
                                "id": "${new}"
                            }]
                        },
                        "baseTerritory": {
                            "data": { "type": "territories", "id": "USA" }
                        }
                    }
                },
                "included": [{
                    "type": "appPrices",
                    "id": "${new}",
                    "attributes": attrs,
                    "relationships": {
                        "appPricePoint": {
                            "data": {
                                "type": "appPricePoints",
                                "id": price_point
                            }
                        }
                    }
                }]
            });
            client.post("/appPriceSchedules", &body).await
        }
    }
}
