use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum InAppCommand {
    /// Manage in-app products (managed products / one-time)
    Products {
        #[command(subcommand)]
        command: ProductsCommand,
    },
    /// Manage subscriptions (v2 monetization API)
    Subscriptions {
        #[command(subcommand)]
        command: SubscriptionsCommand,
    },
}

#[derive(Subcommand)]
pub enum ProductsCommand {
    /// List in-app products
    List {
        /// Package name
        package_name: String,
    },
    /// Get in-app product details
    Get {
        /// Package name
        package_name: String,
        /// SKU / product ID
        #[arg(long)]
        sku: String,
    },
    /// Create an in-app product
    Create {
        /// Package name
        package_name: String,
        /// SKU / product ID
        #[arg(long)]
        sku: String,
        /// Default price in micros (e.g., 990000 = $0.99)
        #[arg(long)]
        price_micros: String,
        /// Currency code (e.g., USD)
        #[arg(long)]
        currency: String,
        /// Title (default locale)
        #[arg(long)]
        title: String,
        /// Description (default locale)
        #[arg(long)]
        description: String,
        /// Status: active or inactive
        #[arg(long, default_value = "active")]
        status: String,
    },
    /// Update an in-app product
    Update {
        /// Package name
        package_name: String,
        /// SKU / product ID
        #[arg(long)]
        sku: String,
        /// Default price in micros
        #[arg(long)]
        price_micros: Option<String>,
        /// Currency code
        #[arg(long)]
        currency: Option<String>,
        /// Title (default locale)
        #[arg(long)]
        title: Option<String>,
        /// Description (default locale)
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete an in-app product
    Delete {
        /// Package name
        package_name: String,
        /// SKU / product ID
        #[arg(long)]
        sku: String,
    },
}

#[derive(Subcommand)]
pub enum SubscriptionsCommand {
    /// List subscriptions
    List {
        /// Package name
        package_name: String,
    },
    /// Get subscription details
    Get {
        /// Package name
        package_name: String,
        /// Product ID
        #[arg(long)]
        product_id: String,
    },
    /// Create a subscription
    Create {
        /// Package name
        package_name: String,
        /// Product ID
        #[arg(long)]
        product_id: String,
        /// JSON body for the subscription resource
        #[arg(long)]
        body: String,
    },
    /// Archive (soft-delete) a subscription
    Archive {
        /// Package name
        package_name: String,
        /// Product ID
        #[arg(long)]
        product_id: String,
    },
}

pub async fn handle(
    cmd: &InAppCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        InAppCommand::Products { command } => handle_products(command, client).await,
        InAppCommand::Subscriptions { command } => handle_subscriptions(command, client).await,
    }
}

async fn handle_products(
    cmd: &ProductsCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ProductsCommand::List { package_name } => {
            client
                .get(&format!("/{package_name}/inappproducts"), &[])
                .await
        }
        ProductsCommand::Get { package_name, sku } => {
            client
                .get::<Value>(&format!("/{package_name}/inappproducts/{sku}"), &[])
                .await
        }
        ProductsCommand::Create {
            package_name,
            sku,
            price_micros,
            currency,
            title,
            description,
            status,
        } => {
            let body = json!({
                "sku": sku,
                "status": status,
                "purchaseType": "managedUser",
                "defaultPrice": {
                    "priceMicros": price_micros,
                    "currency": currency
                },
                "listings": {
                    "en-US": {
                        "title": title,
                        "description": description
                    }
                }
            });
            client
                .post(&format!("/{package_name}/inappproducts"), &body)
                .await
        }
        ProductsCommand::Update {
            package_name,
            sku,
            price_micros,
            currency,
            title,
            description,
        } => {
            let mut current: Value = client
                .get::<Value>(&format!("/{package_name}/inappproducts/{sku}"), &[])
                .await?;
            if let (Some(pm), Some(c)) = (price_micros, currency) {
                current["defaultPrice"]["priceMicros"] = json!(pm);
                current["defaultPrice"]["currency"] = json!(c);
            }
            if let Some(t) = title {
                current["listings"]["en-US"]["title"] = json!(t);
            }
            if let Some(d) = description {
                current["listings"]["en-US"]["description"] = json!(d);
            }
            client
                .put(&format!("/{package_name}/inappproducts/{sku}"), &current)
                .await
        }
        ProductsCommand::Delete { package_name, sku } => {
            client
                .delete_path(&format!("/{package_name}/inappproducts/{sku}"))
                .await
        }
    }
}

async fn handle_subscriptions(
    cmd: &SubscriptionsCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        SubscriptionsCommand::List { package_name } => {
            client
                .get(
                    &format!("/{package_name}/subscriptions"),
                    &[],
                )
                .await
        }
        SubscriptionsCommand::Get {
            package_name,
            product_id,
        } => {
            client
                .get::<Value>(
                    &format!("/{package_name}/subscriptions/{product_id}"),
                    &[],
                )
                .await
        }
        SubscriptionsCommand::Create {
            package_name,
            product_id,
            body,
        } => {
            let body_json: Value = serde_json::from_str(body)?;
            client
                .post(
                    &format!("/{package_name}/subscriptions?productId={product_id}"),
                    &body_json,
                )
                .await
        }
        SubscriptionsCommand::Archive {
            package_name,
            product_id,
        } => {
            client
                .post(
                    &format!("/{package_name}/subscriptions/{product_id}:archive"),
                    &json!({}),
                )
                .await
        }
    }
}
