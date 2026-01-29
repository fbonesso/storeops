use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum DevicesCommand {
    /// List registered devices
    List {
        /// Filter by platform
        #[arg(long)]
        platform: Option<String>,
    },
    /// Register a new device
    Register {
        /// Device name
        #[arg(long)]
        name: String,
        /// Device UDID
        #[arg(long)]
        udid: String,
        /// Platform (ios, macos)
        #[arg(long)]
        platform: String,
    },
}

pub async fn handle(
    cmd: &DevicesCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        DevicesCommand::List { platform } => {
            let mut query = vec![];
            let limit_str = limit.unwrap_or(50).to_string();
            query.push(("limit", limit_str.as_str()));
            let platform_upper;
            if let Some(p) = platform {
                platform_upper = p.to_uppercase();
                query.push(("filter[platform]", &platform_upper));
            }
            client.get("/devices", &query).await
        }
        DevicesCommand::Register {
            name,
            udid,
            platform,
        } => {
            let body = json!({
                "data": {
                    "type": "devices",
                    "attributes": {
                        "name": name,
                        "udid": udid,
                        "platform": platform.to_uppercase()
                    }
                }
            });
            client.post("/devices", &body).await
        }
    }
}
