use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum VersionsCommand {
    /// List app versions
    List {
        /// App ID
        app_id: String,
    },
    /// Create a new version
    Create {
        /// App ID
        app_id: String,
        /// Version string (e.g., "1.2.0")
        #[arg(long)]
        version: String,
    },
}

pub async fn handle(
    cmd: &VersionsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        VersionsCommand::List { app_id } => {
            let mut query = vec![];
            let limit_str = limit.unwrap_or(50).to_string();
            query.push(("limit", limit_str.as_str()));
            client
                .get(&format!("/apps/{app_id}/appStoreVersions"), &query)
                .await
        }
        VersionsCommand::Create { app_id, version } => {
            let body = json!({
                "data": {
                    "type": "appStoreVersions",
                    "attributes": {
                        "versionString": version,
                        "platform": "IOS"
                    },
                    "relationships": {
                        "app": {
                            "data": {
                                "type": "apps",
                                "id": app_id
                            }
                        }
                    }
                }
            });
            client.post("/appStoreVersions", &body).await
        }
    }
}
