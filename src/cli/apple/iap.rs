use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum IapCommand {
    /// List in-app purchases for an app
    List {
        /// App ID
        app_id: String,
    },
    /// Get in-app purchase details
    Get {
        /// In-App Purchase ID
        iap_id: String,
    },
    /// Create an in-app purchase
    Create {
        /// App ID
        app_id: String,
        /// Product name
        #[arg(long)]
        name: String,
        /// Product ID / reference name
        #[arg(long)]
        product_id: String,
        /// Type: CONSUMABLE, NON_CONSUMABLE, NON_RENEWING_SUBSCRIPTION
        #[arg(long)]
        iap_type: String,
    },
    /// Update an in-app purchase
    Update {
        /// In-App Purchase ID
        iap_id: String,
        /// Product name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete an in-app purchase
    Delete {
        /// In-App Purchase ID
        iap_id: String,
    },
    /// Manage in-app purchase localizations
    Localizations {
        #[command(subcommand)]
        command: IapLocalizationsCommand,
    },
    /// Manage in-app purchase price schedules
    Prices {
        #[command(subcommand)]
        command: IapPricesCommand,
    },
    /// Submit an in-app purchase for review
    Submit {
        /// In-App Purchase ID
        iap_id: String,
    },
}

#[derive(Subcommand)]
pub enum IapLocalizationsCommand {
    /// List localizations for an IAP
    List {
        /// In-App Purchase ID
        iap_id: String,
    },
    /// Create a localization
    Create {
        /// In-App Purchase ID
        iap_id: String,
        /// Locale
        #[arg(long)]
        locale: String,
        /// Display name
        #[arg(long)]
        name: String,
        /// Description
        #[arg(long)]
        description: String,
    },
    /// Update a localization
    Update {
        /// Localization ID
        localization_id: String,
        /// Display name
        #[arg(long)]
        name: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a localization
    Delete {
        /// Localization ID
        localization_id: String,
    },
}

#[derive(Subcommand)]
pub enum IapPricesCommand {
    /// Get price schedule for an IAP
    Get {
        /// In-App Purchase ID
        iap_id: String,
    },
    /// List available price points
    Points {
        /// In-App Purchase ID
        iap_id: String,
        /// Filter by territory
        #[arg(long)]
        territory: Option<String>,
    },
}

pub async fn handle(
    cmd: &IapCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        IapCommand::List { app_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/apps/{app_id}/inAppPurchasesV2"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        IapCommand::Get { iap_id } => {
            client
                .get::<Value>(&format!("/inAppPurchasesV2/{iap_id}"), &[])
                .await
        }
        IapCommand::Create {
            app_id,
            name,
            product_id,
            iap_type,
        } => {
            let body = json!({
                "data": {
                    "type": "inAppPurchases",
                    "attributes": {
                        "name": name,
                        "productId": product_id,
                        "inAppPurchaseType": iap_type
                    },
                    "relationships": {
                        "app": {
                            "data": { "type": "apps", "id": app_id }
                        }
                    }
                }
            });
            client.post("/inAppPurchasesV2", &body).await
        }
        IapCommand::Update { iap_id, name } => {
            let mut attrs = json!({});
            if let Some(n) = name {
                attrs["name"] = json!(n);
            }
            let body = json!({
                "data": {
                    "type": "inAppPurchases",
                    "id": iap_id,
                    "attributes": attrs
                }
            });
            client
                .patch(&format!("/inAppPurchasesV2/{iap_id}"), &body)
                .await
        }
        IapCommand::Delete { iap_id } => {
            client
                .delete(&format!("/inAppPurchasesV2/{iap_id}"))
                .await
        }
        IapCommand::Localizations { command } => {
            handle_iap_localizations(command, client, limit).await
        }
        IapCommand::Prices { command } => handle_iap_prices(command, client, limit).await,
        IapCommand::Submit { iap_id } => {
            let body = json!({
                "data": {
                    "type": "inAppPurchaseSubmissions",
                    "relationships": {
                        "inAppPurchaseV2": {
                            "data": {
                                "type": "inAppPurchases",
                                "id": iap_id
                            }
                        }
                    }
                }
            });
            client.post("/inAppPurchaseSubmissions", &body).await
        }
    }
}

async fn handle_iap_localizations(
    cmd: &IapLocalizationsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        IapLocalizationsCommand::List { iap_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/inAppPurchasesV2/{iap_id}/inAppPurchaseLocalizations"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        IapLocalizationsCommand::Create {
            iap_id,
            locale,
            name,
            description,
        } => {
            let body = json!({
                "data": {
                    "type": "inAppPurchaseLocalizations",
                    "attributes": {
                        "locale": locale,
                        "name": name,
                        "description": description
                    },
                    "relationships": {
                        "inAppPurchaseV2": {
                            "data": {
                                "type": "inAppPurchases",
                                "id": iap_id
                            }
                        }
                    }
                }
            });
            client.post("/inAppPurchaseLocalizations", &body).await
        }
        IapLocalizationsCommand::Update {
            localization_id,
            name,
            description,
        } => {
            let mut attrs = json!({});
            if let Some(n) = name {
                attrs["name"] = json!(n);
            }
            if let Some(d) = description {
                attrs["description"] = json!(d);
            }
            let body = json!({
                "data": {
                    "type": "inAppPurchaseLocalizations",
                    "id": localization_id,
                    "attributes": attrs
                }
            });
            client
                .patch(
                    &format!("/inAppPurchaseLocalizations/{localization_id}"),
                    &body,
                )
                .await
        }
        IapLocalizationsCommand::Delete { localization_id } => {
            client
                .delete(&format!(
                    "/inAppPurchaseLocalizations/{localization_id}"
                ))
                .await
        }
    }
}

async fn handle_iap_prices(
    cmd: &IapPricesCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        IapPricesCommand::Get { iap_id } => {
            client
                .get::<Value>(
                    &format!("/inAppPurchasesV2/{iap_id}/iapPriceSchedule"),
                    &[("include", "manualPrices,automaticPrices")],
                )
                .await
        }
        IapPricesCommand::Points { iap_id, territory } => {
            let limit_str = limit.unwrap_or(50).to_string();
            let mut query = vec![("limit", limit_str.as_str())];
            let territory_val;
            if let Some(t) = territory {
                territory_val = t.clone();
                query.push(("filter[territory]", &territory_val));
            }
            client
                .get(
                    &format!("/inAppPurchasesV2/{iap_id}/pricePoints"),
                    &query,
                )
                .await
        }
    }
}
