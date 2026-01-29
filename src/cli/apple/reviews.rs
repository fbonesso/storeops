use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum ReviewsCommand {
    /// List customer reviews
    List {
        /// App ID
        app_id: String,
        /// Filter by rating (1-5)
        #[arg(long)]
        rating: Option<u8>,
        /// Sort order
        #[arg(long, default_value = "recent")]
        sort: String,
    },
    /// Respond to a review
    Respond {
        /// Review ID
        review_id: String,
        /// Response text
        #[arg(long)]
        body: String,
    },
}

pub async fn handle(
    cmd: &ReviewsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ReviewsCommand::List {
            app_id,
            rating,
            sort,
        } => {
            let mut query = vec![];
            let limit_str = limit.unwrap_or(50).to_string();
            query.push(("limit", limit_str.as_str()));
            let sort_value = match sort.as_str() {
                "helpful" => "-rating",
                _ => "-createdDate",
            };
            query.push(("sort", sort_value));
            let rating_str;
            if let Some(r) = rating {
                rating_str = r.to_string();
                query.push(("filter[rating]", &rating_str));
            }
            client
                .get(&format!("/apps/{app_id}/customerReviews"), &query)
                .await
        }
        ReviewsCommand::Respond { review_id, body } => {
            let payload = json!({
                "data": {
                    "type": "customerReviewResponses",
                    "attributes": {
                        "responseBody": body
                    },
                    "relationships": {
                        "review": {
                            "data": {
                                "type": "customerReviews",
                                "id": review_id
                            }
                        }
                    }
                }
            });
            client.post("/customerReviewResponses", &payload).await
        }
    }
}
