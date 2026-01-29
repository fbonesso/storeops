use clap::Subcommand;
use serde_json::Value;

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum AnalyticsCommand {
    /// Download sales and trends report
    Sales {
        /// App ID (vendor number)
        app_id: String,
        /// Report period
        #[arg(long, default_value = "daily")]
        period: String,
    },
}

pub async fn handle(
    cmd: &AnalyticsCommand,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AnalyticsCommand::Sales { app_id, period } => {
            let frequency = match period.as_str() {
                "weekly" => "WEEKLY",
                "monthly" => "MONTHLY",
                _ => "DAILY",
            };
            client
                .get(
                    "/salesReports",
                    &[
                        ("filter[vendorNumber]", app_id.as_str()),
                        ("filter[frequency]", frequency),
                        ("filter[reportType]", "SALES"),
                        ("filter[reportSubType]", "SUMMARY"),
                    ],
                )
                .await
        }
    }
}
