use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum MetadataCommand {
    /// Manage version localizations (description, keywords, what's new, etc.)
    Localizations {
        #[command(subcommand)]
        command: LocalizationsCommand,
    },
    /// Manage app-level info localizations (subtitle, privacy text)
    AppInfo {
        #[command(subcommand)]
        command: AppInfoCommand,
    },
    /// Manage app categories
    Categories {
        #[command(subcommand)]
        command: CategoriesCommand,
    },
}

#[derive(Subcommand)]
pub enum LocalizationsCommand {
    /// List all localizations for a version
    List {
        /// App Store Version ID
        version_id: String,
    },
    /// Get a specific localization
    Get {
        /// Localization ID
        localization_id: String,
    },
    /// Create a localization for a version
    Create {
        /// App Store Version ID
        version_id: String,
        /// Locale (e.g., en-US, pt-BR, ja)
        #[arg(long)]
        locale: String,
        /// App description
        #[arg(long)]
        description: Option<String>,
        /// Keywords (comma-separated)
        #[arg(long)]
        keywords: Option<String>,
        /// What's new text
        #[arg(long)]
        whats_new: Option<String>,
        /// Promotional text
        #[arg(long)]
        promo_text: Option<String>,
        /// Marketing URL
        #[arg(long)]
        marketing_url: Option<String>,
        /// Support URL
        #[arg(long)]
        support_url: Option<String>,
    },
    /// Update a localization
    Update {
        /// Localization ID
        localization_id: String,
        /// App description
        #[arg(long)]
        description: Option<String>,
        /// Keywords (comma-separated)
        #[arg(long)]
        keywords: Option<String>,
        /// What's new text
        #[arg(long)]
        whats_new: Option<String>,
        /// Promotional text
        #[arg(long)]
        promo_text: Option<String>,
        /// Marketing URL
        #[arg(long)]
        marketing_url: Option<String>,
        /// Support URL
        #[arg(long)]
        support_url: Option<String>,
    },
    /// Delete a localization
    Delete {
        /// Localization ID
        localization_id: String,
    },
}

#[derive(Subcommand)]
pub enum AppInfoCommand {
    /// List app info localizations
    List {
        /// App Info ID
        app_info_id: String,
    },
    /// Create app info localization (subtitle, privacy text)
    Create {
        /// App Info ID
        app_info_id: String,
        /// Locale (e.g., en-US)
        #[arg(long)]
        locale: String,
        /// App subtitle
        #[arg(long)]
        subtitle: Option<String>,
        /// Privacy policy text
        #[arg(long)]
        privacy_text: Option<String>,
        /// Privacy policy URL
        #[arg(long)]
        privacy_url: Option<String>,
    },
    /// Update app info localization
    Update {
        /// App Info Localization ID
        localization_id: String,
        /// App subtitle
        #[arg(long)]
        subtitle: Option<String>,
        /// Privacy policy text
        #[arg(long)]
        privacy_text: Option<String>,
        /// Privacy policy URL
        #[arg(long)]
        privacy_url: Option<String>,
    },
    /// Delete app info localization
    Delete {
        /// App Info Localization ID
        localization_id: String,
    },
}

#[derive(Subcommand)]
pub enum CategoriesCommand {
    /// List all available categories
    List,
    /// Get current categories for an app
    Get {
        /// App Info ID
        app_info_id: String,
    },
    /// Set primary and secondary categories
    Set {
        /// App Info ID
        app_info_id: String,
        /// Primary category ID
        #[arg(long)]
        primary: String,
        /// Secondary category ID (optional)
        #[arg(long)]
        secondary: Option<String>,
    },
}

pub async fn handle(
    cmd: &MetadataCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        MetadataCommand::Localizations { command } => {
            handle_localizations(command, client, limit).await
        }
        MetadataCommand::AppInfo { command } => handle_app_info(command, client, limit).await,
        MetadataCommand::Categories { command } => handle_categories(command, client, limit).await,
    }
}

async fn handle_localizations(
    cmd: &LocalizationsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        LocalizationsCommand::List { version_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/appStoreVersions/{version_id}/appStoreVersionLocalizations"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        LocalizationsCommand::Get { localization_id } => {
            client
                .get::<Value>(
                    &format!("/appStoreVersionLocalizations/{localization_id}"),
                    &[],
                )
                .await
        }
        LocalizationsCommand::Create {
            version_id,
            locale,
            description,
            keywords,
            whats_new,
            promo_text,
            marketing_url,
            support_url,
        } => {
            let mut attrs = json!({ "locale": locale });
            if let Some(v) = description {
                attrs["description"] = json!(v);
            }
            if let Some(v) = keywords {
                attrs["keywords"] = json!(v);
            }
            if let Some(v) = whats_new {
                attrs["whatsNew"] = json!(v);
            }
            if let Some(v) = promo_text {
                attrs["promotionalText"] = json!(v);
            }
            if let Some(v) = marketing_url {
                attrs["marketingUrl"] = json!(v);
            }
            if let Some(v) = support_url {
                attrs["supportUrl"] = json!(v);
            }
            let body = json!({
                "data": {
                    "type": "appStoreVersionLocalizations",
                    "attributes": attrs,
                    "relationships": {
                        "appStoreVersion": {
                            "data": {
                                "type": "appStoreVersions",
                                "id": version_id
                            }
                        }
                    }
                }
            });
            client.post("/appStoreVersionLocalizations", &body).await
        }
        LocalizationsCommand::Update {
            localization_id,
            description,
            keywords,
            whats_new,
            promo_text,
            marketing_url,
            support_url,
        } => {
            let mut attrs = json!({});
            if let Some(v) = description {
                attrs["description"] = json!(v);
            }
            if let Some(v) = keywords {
                attrs["keywords"] = json!(v);
            }
            if let Some(v) = whats_new {
                attrs["whatsNew"] = json!(v);
            }
            if let Some(v) = promo_text {
                attrs["promotionalText"] = json!(v);
            }
            if let Some(v) = marketing_url {
                attrs["marketingUrl"] = json!(v);
            }
            if let Some(v) = support_url {
                attrs["supportUrl"] = json!(v);
            }
            let body = json!({
                "data": {
                    "type": "appStoreVersionLocalizations",
                    "id": localization_id,
                    "attributes": attrs
                }
            });
            client
                .patch(
                    &format!("/appStoreVersionLocalizations/{localization_id}"),
                    &body,
                )
                .await
        }
        LocalizationsCommand::Delete { localization_id } => {
            client
                .delete(&format!("/appStoreVersionLocalizations/{localization_id}"))
                .await
        }
    }
}

async fn handle_app_info(
    cmd: &AppInfoCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AppInfoCommand::List { app_info_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/appInfos/{app_info_id}/appInfoLocalizations"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        AppInfoCommand::Create {
            app_info_id,
            locale,
            subtitle,
            privacy_text,
            privacy_url,
        } => {
            let mut attrs = json!({ "locale": locale });
            if let Some(v) = subtitle {
                attrs["subtitle"] = json!(v);
            }
            if let Some(v) = privacy_text {
                attrs["privacyPolicyText"] = json!(v);
            }
            if let Some(v) = privacy_url {
                attrs["privacyPolicyUrl"] = json!(v);
            }
            let body = json!({
                "data": {
                    "type": "appInfoLocalizations",
                    "attributes": attrs,
                    "relationships": {
                        "appInfo": {
                            "data": {
                                "type": "appInfos",
                                "id": app_info_id
                            }
                        }
                    }
                }
            });
            client.post("/appInfoLocalizations", &body).await
        }
        AppInfoCommand::Update {
            localization_id,
            subtitle,
            privacy_text,
            privacy_url,
        } => {
            let mut attrs = json!({});
            if let Some(v) = subtitle {
                attrs["subtitle"] = json!(v);
            }
            if let Some(v) = privacy_text {
                attrs["privacyPolicyText"] = json!(v);
            }
            if let Some(v) = privacy_url {
                attrs["privacyPolicyUrl"] = json!(v);
            }
            let body = json!({
                "data": {
                    "type": "appInfoLocalizations",
                    "id": localization_id,
                    "attributes": attrs
                }
            });
            client
                .patch(&format!("/appInfoLocalizations/{localization_id}"), &body)
                .await
        }
        AppInfoCommand::Delete { localization_id } => {
            client
                .delete(&format!("/appInfoLocalizations/{localization_id}"))
                .await
        }
    }
}

async fn handle_categories(
    cmd: &CategoriesCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        CategoriesCommand::List => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get("/appCategories", &[("limit", limit_str.as_str())])
                .await
        }
        CategoriesCommand::Get { app_info_id } => {
            client
                .get::<Value>(
                    &format!("/appInfos/{app_info_id}"),
                    &[("include", "primaryCategory,secondaryCategory")],
                )
                .await
        }
        CategoriesCommand::Set {
            app_info_id,
            primary,
            secondary,
        } => {
            let mut relationships = json!({
                "primaryCategory": {
                    "data": {
                        "type": "appCategories",
                        "id": primary
                    }
                }
            });
            if let Some(sec) = secondary {
                relationships["secondaryCategory"] = json!({
                    "data": {
                        "type": "appCategories",
                        "id": sec
                    }
                });
            }
            let body = json!({
                "data": {
                    "type": "appInfos",
                    "id": app_info_id,
                    "relationships": relationships
                }
            });
            client
                .patch(&format!("/appInfos/{app_info_id}"), &body)
                .await
        }
    }
}
