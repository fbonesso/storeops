use clap::Subcommand;
use serde_json::Value;

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum BuildsCommand {
    /// List builds for an app
    List {
        /// App ID
        app_id: String,
    },
    /// Get build details
    Info {
        /// Build ID
        build_id: String,
    },
}

pub async fn handle(
    cmd: &BuildsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        BuildsCommand::List { app_id } => {
            let mut query = vec![];
            let limit_str = limit.unwrap_or(50).to_string();
            query.push(("limit", limit_str.as_str()));
            query.push(("filter[app]", app_id.as_str()));
            client.get("/builds", &query).await
        }
        BuildsCommand::Info { build_id } => {
            client
                .get::<Value>(&format!("/builds/{build_id}"), &[])
                .await
        }
    }
}
