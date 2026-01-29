use clap::Subcommand;
use serde_json::Value;

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum AppsCommand {
    /// List all apps
    List,
    /// Get app details
    Info {
        /// App ID
        app_id: String,
    },
}

pub async fn handle(
    cmd: &AppsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AppsCommand::List => {
            let mut query = vec![];
            let limit_str = limit.unwrap_or(50).to_string();
            query.push(("limit", limit_str.as_str()));
            client.get("/apps", &query).await
        }
        AppsCommand::Info { app_id } => client.get::<Value>(&format!("/apps/{app_id}"), &[]).await,
    }
}
