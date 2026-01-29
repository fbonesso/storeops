use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum SubscriptionsCommand {
    /// Manage subscription groups
    Groups {
        #[command(subcommand)]
        command: GroupsCommand,
    },
    /// Manage subscriptions within a group
    Items {
        #[command(subcommand)]
        command: ItemsCommand,
    },
    /// Manage subscription localizations
    Localizations {
        #[command(subcommand)]
        command: SubLocalizationsCommand,
    },
    /// Manage subscription price schedules
    Prices {
        #[command(subcommand)]
        command: SubPricesCommand,
    },
    /// Manage promotional offers
    Offers {
        #[command(subcommand)]
        command: OffersCommand,
    },
}

#[derive(Subcommand)]
pub enum GroupsCommand {
    /// List subscription groups for an app
    List {
        /// App ID
        app_id: String,
    },
    /// Create a subscription group
    Create {
        /// App ID
        app_id: String,
        /// Group reference name
        #[arg(long)]
        name: String,
    },
    /// Update a subscription group
    Update {
        /// Subscription Group ID
        group_id: String,
        /// New reference name
        #[arg(long)]
        name: String,
    },
    /// Delete a subscription group
    Delete {
        /// Subscription Group ID
        group_id: String,
    },
}

#[derive(Subcommand)]
pub enum ItemsCommand {
    /// List subscriptions in a group
    List {
        /// Subscription Group ID
        group_id: String,
    },
    /// Create a subscription
    Create {
        /// Subscription Group ID
        group_id: String,
        /// Product ID
        #[arg(long)]
        product_id: String,
        /// Reference name
        #[arg(long)]
        name: String,
        /// Duration: ONE_WEEK, ONE_MONTH, TWO_MONTHS, THREE_MONTHS, SIX_MONTHS, ONE_YEAR
        #[arg(long)]
        duration: String,
    },
    /// Get subscription details
    Get {
        /// Subscription ID
        subscription_id: String,
    },
    /// Delete a subscription
    Delete {
        /// Subscription ID
        subscription_id: String,
    },
}

#[derive(Subcommand)]
pub enum SubLocalizationsCommand {
    /// List localizations for a subscription
    List {
        /// Subscription ID
        subscription_id: String,
    },
    /// Create a localization
    Create {
        /// Subscription ID
        subscription_id: String,
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
pub enum SubPricesCommand {
    /// Get price schedule for a subscription
    Get {
        /// Subscription ID
        subscription_id: String,
    },
    /// List available price points
    Points {
        /// Subscription ID
        subscription_id: String,
        /// Filter by territory
        #[arg(long)]
        territory: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum OffersCommand {
    /// List promotional offers for a subscription
    List {
        /// Subscription ID
        subscription_id: String,
    },
    /// Create a promotional offer
    Create {
        /// Subscription ID
        subscription_id: String,
        /// Offer name / reference
        #[arg(long)]
        name: String,
        /// Offer code
        #[arg(long)]
        offer_code: String,
        /// Duration: ONE_WEEK, ONE_MONTH, etc.
        #[arg(long)]
        duration: String,
        /// Number of periods
        #[arg(long)]
        periods: u32,
        /// Offer mode: PAY_AS_YOU_GO, PAY_UP_FRONT, FREE_TRIAL
        #[arg(long)]
        mode: String,
    },
    /// Delete a promotional offer
    Delete {
        /// Offer ID
        offer_id: String,
    },
}

pub async fn handle(
    cmd: &SubscriptionsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        SubscriptionsCommand::Groups { command } => handle_groups(command, client, limit).await,
        SubscriptionsCommand::Items { command } => handle_items(command, client, limit).await,
        SubscriptionsCommand::Localizations { command } => {
            handle_localizations(command, client, limit).await
        }
        SubscriptionsCommand::Prices { command } => handle_prices(command, client, limit).await,
        SubscriptionsCommand::Offers { command } => handle_offers(command, client, limit).await,
    }
}

async fn handle_groups(
    cmd: &GroupsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        GroupsCommand::List { app_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/apps/{app_id}/subscriptionGroups"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        GroupsCommand::Create { app_id, name } => {
            let body = json!({
                "data": {
                    "type": "subscriptionGroups",
                    "attributes": { "referenceName": name },
                    "relationships": {
                        "app": { "data": { "type": "apps", "id": app_id } }
                    }
                }
            });
            client.post("/subscriptionGroups", &body).await
        }
        GroupsCommand::Update { group_id, name } => {
            let body = json!({
                "data": {
                    "type": "subscriptionGroups",
                    "id": group_id,
                    "attributes": { "referenceName": name }
                }
            });
            client
                .patch(&format!("/subscriptionGroups/{group_id}"), &body)
                .await
        }
        GroupsCommand::Delete { group_id } => {
            client
                .delete(&format!("/subscriptionGroups/{group_id}"))
                .await
        }
    }
}

async fn handle_items(
    cmd: &ItemsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ItemsCommand::List { group_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/subscriptionGroups/{group_id}/subscriptions"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        ItemsCommand::Create {
            group_id,
            product_id,
            name,
            duration,
        } => {
            let body = json!({
                "data": {
                    "type": "subscriptions",
                    "attributes": {
                        "productId": product_id,
                        "name": name,
                        "subscriptionPeriod": duration
                    },
                    "relationships": {
                        "group": {
                            "data": { "type": "subscriptionGroups", "id": group_id }
                        }
                    }
                }
            });
            client.post("/subscriptions", &body).await
        }
        ItemsCommand::Get { subscription_id } => {
            client
                .get::<Value>(&format!("/subscriptions/{subscription_id}"), &[])
                .await
        }
        ItemsCommand::Delete { subscription_id } => {
            client
                .delete(&format!("/subscriptions/{subscription_id}"))
                .await
        }
    }
}

async fn handle_localizations(
    cmd: &SubLocalizationsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        SubLocalizationsCommand::List { subscription_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/subscriptions/{subscription_id}/subscriptionLocalizations"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        SubLocalizationsCommand::Create {
            subscription_id,
            locale,
            name,
            description,
        } => {
            let body = json!({
                "data": {
                    "type": "subscriptionLocalizations",
                    "attributes": {
                        "locale": locale,
                        "name": name,
                        "description": description
                    },
                    "relationships": {
                        "subscription": {
                            "data": { "type": "subscriptions", "id": subscription_id }
                        }
                    }
                }
            });
            client.post("/subscriptionLocalizations", &body).await
        }
        SubLocalizationsCommand::Update {
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
                    "type": "subscriptionLocalizations",
                    "id": localization_id,
                    "attributes": attrs
                }
            });
            client
                .patch(
                    &format!("/subscriptionLocalizations/{localization_id}"),
                    &body,
                )
                .await
        }
        SubLocalizationsCommand::Delete { localization_id } => {
            client
                .delete(&format!(
                    "/subscriptionLocalizations/{localization_id}"
                ))
                .await
        }
    }
}

async fn handle_prices(
    cmd: &SubPricesCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        SubPricesCommand::Get { subscription_id } => {
            client
                .get::<Value>(
                    &format!("/subscriptions/{subscription_id}/pricePoints"),
                    &[],
                )
                .await
        }
        SubPricesCommand::Points {
            subscription_id,
            territory,
        } => {
            let limit_str = limit.unwrap_or(50).to_string();
            let mut query = vec![("limit", limit_str.as_str())];
            let territory_val;
            if let Some(t) = territory {
                territory_val = t.clone();
                query.push(("filter[territory]", &territory_val));
            }
            client
                .get(
                    &format!("/subscriptions/{subscription_id}/pricePoints"),
                    &query,
                )
                .await
        }
    }
}

async fn handle_offers(
    cmd: &OffersCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        OffersCommand::List { subscription_id } => {
            let limit_str = limit.unwrap_or(50).to_string();
            client
                .get(
                    &format!("/subscriptions/{subscription_id}/promotionalOffers"),
                    &[("limit", limit_str.as_str())],
                )
                .await
        }
        OffersCommand::Create {
            subscription_id,
            name,
            offer_code,
            duration,
            periods,
            mode,
        } => {
            let body = json!({
                "data": {
                    "type": "subscriptionPromotionalOffers",
                    "attributes": {
                        "name": name,
                        "offerCode": offer_code,
                        "duration": duration,
                        "numberOfPeriods": periods,
                        "offerMode": mode
                    },
                    "relationships": {
                        "subscription": {
                            "data": { "type": "subscriptions", "id": subscription_id }
                        }
                    }
                }
            });
            client
                .post("/subscriptionPromotionalOffers", &body)
                .await
        }
        OffersCommand::Delete { offer_id } => {
            client
                .delete(&format!("/subscriptionPromotionalOffers/{offer_id}"))
                .await
        }
    }
}
