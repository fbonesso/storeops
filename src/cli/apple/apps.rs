use clap::Subcommand;
use serde_json::Value;

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum AppsCommand {
    /// List all apps available in App Store Connect
    ///
    /// Examples:
    ///   storeops apple apps list
    ///   storeops apple apps list --limit 10
    List,
    /// Get detailed information for a specific app
    ///
    /// Examples:
    ///   storeops apple apps info --app-id 1234567890
    Info {
        /// Your App Store Connect app ID (not the bundle ID)
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
