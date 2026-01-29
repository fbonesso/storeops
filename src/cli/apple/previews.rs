use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum PreviewsCommand {
    /// Manage preview sets
    Sets {
        #[command(subcommand)]
        command: PreviewSetsCommand,
    },
    /// Manage individual preview videos
    Videos {
        #[command(subcommand)]
        command: PreviewVideosCommand,
    },
}

#[derive(Subcommand)]
pub enum PreviewSetsCommand {
    /// List app preview sets for a localization
    List {
        /// App Store Version Localization ID
        localization_id: String,
    },
    /// Create a preview set
    Create {
        /// App Store Version Localization ID
        localization_id: String,
        /// Preview type (e.g., IPHONE_67, IPAD_PRO_129)
        #[arg(long)]
        preview_type: String,
    },
    /// Delete a preview set
    Delete {
        /// Preview Set ID
        set_id: String,
    },
}

#[derive(Subcommand)]
pub enum PreviewVideosCommand {
    /// List previews in a set
    List {
        /// Preview Set ID
        set_id: String,
    },
    /// Upload an app preview video
    Upload {
        /// Preview Set ID
        set_id: String,
        /// Path to video file
        #[arg(long)]
        file: String,
        /// Filename for the upload
        #[arg(long)]
        filename: String,
        /// MIME type (e.g., video/mp4)
        #[arg(long, default_value = "video/mp4")]
        mime_type: String,
    },
    /// Delete a preview
    Delete {
        /// Preview ID
        preview_id: String,
    },
}

pub async fn handle(
    cmd: &PreviewsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        PreviewsCommand::Sets { command } => handle_sets(command, client, limit).await,
        PreviewsCommand::Videos { command } => handle_videos(command, client).await,
    }
}

async fn handle_sets(
    cmd: &PreviewSetsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        PreviewSetsCommand::List { localization_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!(
                        "/appStoreVersionLocalizations/{localization_id}/appPreviewSets"
                    ),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        PreviewSetsCommand::Create {
            localization_id,
            preview_type,
        } => {
            let body = json!({
                "data": {
                    "type": "appPreviewSets",
                    "attributes": {
                        "previewType": preview_type
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
            client.post("/appPreviewSets", &body).await
        }
        PreviewSetsCommand::Delete { set_id } => {
            client
                .delete(&format!("/appPreviewSets/{set_id}"))
                .await
        }
    }
}

async fn handle_videos(
    cmd: &PreviewVideosCommand,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        PreviewVideosCommand::List { set_id } => {
            client
                .get::<Value>(
                    &format!("/appPreviewSets/{set_id}/appPreviews"),
                    &[],
                )
                .await
        }
        PreviewVideosCommand::Upload {
            set_id,
            file,
            filename,
            mime_type,
        } => {
            let file_size = tokio::fs::metadata(file).await?.len();
            let reservation = json!({
                "data": {
                    "type": "appPreviews",
                    "attributes": {
                        "fileName": filename,
                        "fileSize": file_size,
                        "mimeType": mime_type
                    },
                    "relationships": {
                        "appPreviewSet": {
                            "data": {
                                "type": "appPreviewSets",
                                "id": set_id
                            }
                        }
                    }
                }
            });
            let reserved: Value = client.post("/appPreviews", &reservation).await?;
            let preview_id = reserved["data"]["id"]
                .as_str()
                .ok_or("no preview id in reservation response")?;

            let upload_ops = &reserved["data"]["attributes"]["uploadOperations"];
            let file_bytes = tokio::fs::read(file).await?;

            if let Some(ops) = upload_ops.as_array() {
                for op in ops {
                    let url = op["url"].as_str().ok_or("missing upload url")?;
                    let offset = op["offset"].as_u64().unwrap_or(0) as usize;
                    let length =
                        op["length"].as_u64().unwrap_or(file_bytes.len() as u64) as usize;
                    let chunk =
                        &file_bytes[offset..std::cmp::min(offset + length, file_bytes.len())];

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

            let commit_body = json!({
                "data": {
                    "type": "appPreviews",
                    "id": preview_id,
                    "attributes": {
                        "uploaded": true,
                        "sourceFileChecksum": reserved["data"]["attributes"]["sourceFileChecksum"]
                    }
                }
            });
            client
                .patch(&format!("/appPreviews/{preview_id}"), &commit_body)
                .await
        }
        PreviewVideosCommand::Delete { preview_id } => {
            client
                .delete(&format!("/appPreviews/{preview_id}"))
                .await
        }
    }
}
