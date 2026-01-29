use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum ImagesCommand {
    /// List images for a locale and image type
    List {
        /// Package name
        package_name: String,
        /// Locale (e.g., en-US)
        #[arg(long)]
        locale: String,
        /// Image type: featureGraphic, icon, phoneScreenshots, sevenInchScreenshots,
        /// tenInchScreenshots, tvBanner, tvScreenshots, wearScreenshots
        #[arg(long)]
        image_type: String,
    },
    /// Upload an image
    Upload {
        /// Package name
        package_name: String,
        /// Locale (e.g., en-US)
        #[arg(long)]
        locale: String,
        /// Image type
        #[arg(long)]
        image_type: String,
        /// Path to image file (PNG or JPEG)
        #[arg(long)]
        file: String,
    },
    /// Delete an image
    Delete {
        /// Package name
        package_name: String,
        /// Locale
        #[arg(long)]
        locale: String,
        /// Image type
        #[arg(long)]
        image_type: String,
        /// Image ID
        #[arg(long)]
        image_id: String,
    },
    /// Delete all images of a given type for a locale
    DeleteAll {
        /// Package name
        package_name: String,
        /// Locale
        #[arg(long)]
        locale: String,
        /// Image type
        #[arg(long)]
        image_type: String,
    },
}

pub async fn handle(
    cmd: &ImagesCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ImagesCommand::List {
            package_name,
            locale,
            image_type,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result: Value = client
                .get(
                    &format!(
                        "/{package_name}/edits/{edit_id}/listings/{locale}/{image_type}"
                    ),
                    &[],
                )
                .await?;
            let _ = client.delete_path(&format!("/{package_name}/edits/{edit_id}")).await;
            Ok(result)
        }
        ImagesCommand::Upload {
            package_name,
            locale,
            image_type,
            file,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;

            let result = client
                .upload_image(package_name, edit_id, locale, image_type, file)
                .await?;

            client
                .post(
                    &format!("/{package_name}/edits/{edit_id}:commit"),
                    &json!({}),
                )
                .await?;
            Ok(result)
        }
        ImagesCommand::Delete {
            package_name,
            locale,
            image_type,
            image_id,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result = client
                .delete_path(&format!(
                    "/{package_name}/edits/{edit_id}/listings/{locale}/{image_type}/{image_id}"
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
        ImagesCommand::DeleteAll {
            package_name,
            locale,
            image_type,
        } => {
            let edit: Value = client
                .post(&format!("/{package_name}/edits"), &json!({}))
                .await?;
            let edit_id = edit["id"].as_str().ok_or("no edit id")?;
            let result = client
                .delete_path(&format!(
                    "/{package_name}/edits/{edit_id}/listings/{locale}/{image_type}"
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
