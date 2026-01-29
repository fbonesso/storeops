use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum ListingsCommand {
    /// List all store listings (all locales)
    List {
        /// Package name
        package_name: String,
    },
    /// Get a store listing for a specific locale
    Get {
        /// Package name
        package_name: String,
        /// Locale (e.g., en-US, pt-BR)
        #[arg(long)]
        locale: String,
    },
    /// Create or update a store listing for a locale
    Update {
        /// Package name
        package_name: String,
        /// Locale (e.g., en-US)
        #[arg(long)]
        locale: String,
        /// App title (max 30 chars)
        #[arg(long)]
        title: Option<String>,
        /// Full description (max 4000 chars)
        #[arg(long)]
        full_description: Option<String>,
        /// Short description (max 80 chars)
        #[arg(long)]
        short_description: Option<String>,
        /// Video URL (YouTube)
        #[arg(long)]
        video: Option<String>,
    },
    /// Delete a store listing for a locale
    Delete {
        /// Package name
        package_name: String,
        /// Locale
        #[arg(long)]
        locale: String,
    },
}

pub async fn handle(
    cmd: &ListingsCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ListingsCommand::List { package_name } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result: Value = client
                .get(
                    &format!("/{package_name}/edits/{edit_id}/listings"),
                    &[],
                )
                .await?;
            let _ = client.delete_path(&format!("/{package_name}/edits/{edit_id}")).await;
            Ok(result)
        }
        ListingsCommand::Get {
            package_name,
            locale,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result: Value = client
                .get(
                    &format!("/{package_name}/edits/{edit_id}/listings/{locale}"),
                    &[],
                )
                .await?;
            let _ = client.delete_path(&format!("/{package_name}/edits/{edit_id}")).await;
            Ok(result)
        }
        ListingsCommand::Update {
            package_name,
            locale,
            title,
            full_description,
            short_description,
            video,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;

            let mut body = json!({ "language": locale });
            if let Some(t) = title {
                body["title"] = json!(t);
            }
            if let Some(d) = full_description {
                body["fullDescription"] = json!(d);
            }
            if let Some(s) = short_description {
                body["shortDescription"] = json!(s);
            }
            if let Some(v) = video {
                body["video"] = json!(v);
            }

            let result = client
                .put(
                    &format!("/{package_name}/edits/{edit_id}/listings/{locale}"),
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
        ListingsCommand::Delete {
            package_name,
            locale,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result = client
                .delete_path(&format!(
                    "/{package_name}/edits/{edit_id}/listings/{locale}"
                ))
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
