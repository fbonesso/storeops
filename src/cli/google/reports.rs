use clap::Subcommand;
use serde_json::Value;

use crate::api::google_client::GoogleClient;

#[derive(Subcommand)]
pub enum ReportsCommand {
    /// Get app statistics
    Stats {
        /// Package name
        package_name: String,
        /// Report dimension
        #[arg(long, default_value = "overview")]
        dimension: String,
    },
}

pub async fn handle(
    cmd: &ReportsCommand,
    _client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        ReportsCommand::Stats {
            package_name,
            dimension,
        } => {
            // Google Play Developer API doesn't have a direct stats endpoint in v3.
            // Stats are available via Google Play Console or the Reporting API (GCS buckets).
            Ok(serde_json::json!({
                "message": format!(
                    "Statistics for '{}' (dimension: {}). Use Google Cloud Storage reporting buckets for detailed stats. See: https://developers.google.com/android-publisher#reporting",
                    package_name, dimension
                )
            }))
        }
    }
}
