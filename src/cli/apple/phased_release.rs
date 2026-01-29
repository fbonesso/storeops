use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum PhasedReleaseCommand {
    /// Get phased release status for a version
    Get {
        /// App Store Version ID
        version_id: String,
    },
    /// Enable phased release for a version
    Create {
        /// App Store Version ID
        version_id: String,
    },
    /// Update phased release state (ACTIVE, PAUSE, COMPLETE)
    Update {
        /// Phased Release ID
        release_id: String,
        /// New state: ACTIVE, PAUSE, or COMPLETE
        #[arg(long)]
        state: String,
    },
    /// Delete (cancel) phased release
    Delete {
        /// Phased Release ID
        release_id: String,
    },
}

pub async fn handle(
    cmd: &PhasedReleaseCommand,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        PhasedReleaseCommand::Get { version_id } => {
            client
                .get::<Value>(
                    &format!("/appStoreVersions/{version_id}/appStoreVersionPhasedRelease"),
                    &[],
                )
                .await
        }
        PhasedReleaseCommand::Create { version_id } => {
            let body = json!({
                "data": {
                    "type": "appStoreVersionPhasedReleases",
                    "attributes": {
                        "phasedReleaseState": "ACTIVE"
                    },
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
            client.post("/appStoreVersionPhasedReleases", &body).await
        }
        PhasedReleaseCommand::Update { release_id, state } => {
            let body = json!({
                "data": {
                    "type": "appStoreVersionPhasedReleases",
                    "id": release_id,
                    "attributes": {
                        "phasedReleaseState": state
                    }
                }
            });
            client
                .patch(
                    &format!("/appStoreVersionPhasedReleases/{release_id}"),
                    &body,
                )
                .await
        }
        PhasedReleaseCommand::Delete { release_id } => {
            client
                .delete(&format!("/appStoreVersionPhasedReleases/{release_id}"))
                .await
        }
    }
}
