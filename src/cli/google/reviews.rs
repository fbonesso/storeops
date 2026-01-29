use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum ReviewsCommand {
    /// List reviews
    List {
        /// Package name
        package_name: String,
        /// Sort order
        #[arg(long, default_value = "recent")]
        sort: String,
    },
    /// Reply to a review
    Reply {
        /// Review ID
        review_id: String,
        /// Package name
        #[arg(long)]
        package_name: String,
        /// Reply text
        #[arg(long)]
        body: String,
    },
}

pub async fn handle(
    cmd: &ReviewsCommand,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ReviewsCommand::List { package_name, .. } => {
            client.get(&format!("/{package_name}/reviews"), &[]).await
        }
        ReviewsCommand::Reply {
            review_id,
            package_name,
            body,
        } => {
            client
                .post(
                    &format!("/{package_name}/reviews/{review_id}:reply"),
                    &json!({
                        "replyText": body
                    }),
                )
                .await
        }
    }
}
