use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum ScreenshotsCommand {
    /// List screenshot sets for a localization
    Sets {
        #[command(subcommand)]
        command: SetsCommand,
    },
    /// Manage individual screenshots
    Images {
        #[command(subcommand)]
        command: ImagesCommand,
    },
}

#[derive(Subcommand)]
pub enum SetsCommand {
    /// List screenshot sets for a version localization
    List {
        /// App Store Version Localization ID
        localization_id: String,
    },
    /// Create a screenshot set
    Create {
        /// App Store Version Localization ID
        localization_id: String,
        /// Display type (e.g., APP_IPHONE_67, APP_IPHONE_65, APP_IPAD_PRO_129, etc.)
        #[arg(long)]
        display_type: String,
    },
    /// Delete a screenshot set
    Delete {
        /// Screenshot Set ID
        set_id: String,
    },
}

#[derive(Subcommand)]
pub enum ImagesCommand {
    /// List screenshots in a set
    List {
        /// Screenshot Set ID
        set_id: String,
    },
    /// Upload a screenshot to a set
    Upload {
        /// Screenshot Set ID
        set_id: String,
        /// Path to image file
        #[arg(long)]
        file: String,
        /// Filename for the upload
        #[arg(long)]
        filename: String,
    },
    /// Delete a screenshot
    Delete {
        /// Screenshot ID
        screenshot_id: String,
    },
    /// Reorder screenshots within a set
    Reorder {
        /// Screenshot Set ID
        set_id: String,
        /// Ordered screenshot IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        ids: Vec<String>,
    },
}

pub async fn handle(
    cmd: &ScreenshotsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ScreenshotsCommand::Sets { command } => handle_sets(command, client, limit).await,
        ScreenshotsCommand::Images { command } => handle_images(command, client, limit).await,
    }
}

async fn handle_sets(
    cmd: &SetsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        SetsCommand::List { localization_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/appStoreVersionLocalizations/{localization_id}/appScreenshotSets"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        SetsCommand::Create {
            localization_id,
            display_type,
        } => {
            let body = json!({
                "data": {
                    "type": "appScreenshotSets",
                    "attributes": {
                        "screenshotDisplayType": display_type
                    },
                    "relationships": {
                        "appStoreVersionLocalization": {
                            "data": {
                                "type": "appStoreVersionLocalizations",
                                "id": localization_id
                            }
                        }
                    }
                }
            });
            client.post("/appScreenshotSets", &body).await
        }
        SetsCommand::Delete { set_id } => {
            client
                .delete(&format!("/appScreenshotSets/{set_id}"))
                .await
        }
    }
}

async fn handle_images(
    cmd: &ImagesCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ImagesCommand::List { set_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/appScreenshotSets/{set_id}/appScreenshots"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        ImagesCommand::Upload {
            set_id,
            file,
            filename,
        } => {
            let file_size = tokio::fs::metadata(file).await?.len();
            // Step 1: Reserve the screenshot
            let reservation = json!({
                "data": {
                    "type": "appScreenshots",
                    "attributes": {
                        "fileName": filename,
                        "fileSize": file_size
                    },
                    "relationships": {
                        "appScreenshotSet": {
                            "data": {
                                "type": "appScreenshotSets",
                                "id": set_id
                            }
                        }
                    }
                }
            });
            let reserved: Value = client.post("/appScreenshots", &reservation).await?;
            let screenshot_id = reserved["data"]["id"]
                .as_str()
                .ok_or("no screenshot id in reservation response")?;

            // Step 2: Upload the asset
            let upload_ops = &reserved["data"]["attributes"]["uploadOperations"];
            let file_bytes = tokio::fs::read(file).await?;

            if let Some(ops) = upload_ops.as_array() {
                for op in ops {
                    let url = op["url"].as_str().ok_or("missing upload url")?;
                    let offset = op["offset"].as_u64().unwrap_or(0) as usize;
                    let length = op["length"].as_u64().unwrap_or(file_bytes.len() as u64) as usize;
                    let chunk = &file_bytes[offset..std::cmp::min(offset + length, file_bytes.len())];

                    let mut req = reqwest::Client::new().put(url);
                    if let Some(headers) = op["requestHeaders"].as_array() {
                        for h in headers {
                            if let (Some(name), Some(value)) =
                                (h["name"].as_str(), h["value"].as_str())
                            {
                                req = req.header(name, value);
                            }
                        }
                    }
                    req.body(chunk.to_vec()).send().await?;
                }
            }

            // Step 3: Commit the upload
            let commit_body = json!({
                "data": {
                    "type": "appScreenshots",
                    "id": screenshot_id,
                    "attributes": {
                        "uploaded": true,
                        "sourceFileChecksum": reserved["data"]["attributes"]["sourceFileChecksum"]
                    }
                }
            });
            client
                .patch(
                    &format!("/appScreenshots/{screenshot_id}"),
                    &commit_body,
                )
                .await
        }
        ImagesCommand::Delete { screenshot_id } => {
            client
                .delete(&format!("/appScreenshots/{screenshot_id}"))
                .await
        }
        ImagesCommand::Reorder { set_id, ids } => {
            let data: Vec<Value> = ids
                .iter()
                .map(|id| {
                    json!({
                        "type": "appScreenshots",
                        "id": id
                    })
                })
                .collect();
            let body = json!({ "data": data });
            client
                .patch(
                    &format!("/appScreenshotSets/{set_id}/relationships/appScreenshots"),
                    &body,
                )
                .await
        }
    }
}
