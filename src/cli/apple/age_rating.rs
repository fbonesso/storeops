use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum AgeRatingCommand {
    /// Get the age rating declaration for an app
    Get {
        /// App ID (will look up the app info automatically)
        app_id: String,
    },
    /// Update age rating declaration
    Update {
        /// Age Rating Declaration ID
        declaration_id: String,
        /// Alcohol, tobacco, or drug use (NONE, INFREQUENT_OR_MILD, FREQUENT_OR_INTENSE)
        #[arg(long)]
        alcohol_tobacco_drugs: Option<String>,
        /// Cartoon or fantasy violence
        #[arg(long)]
        cartoon_violence: Option<String>,
        /// Realistic violence
        #[arg(long)]
        realistic_violence: Option<String>,
        /// Sexual content and nudity
        #[arg(long)]
        sexual_content: Option<String>,
        /// Horror/fear themes
        #[arg(long)]
        horror: Option<String>,
        /// Mature/suggestive themes
        #[arg(long)]
        mature_themes: Option<String>,
        /// Gambling simulated
        #[arg(long)]
        gambling: Option<String>,
        /// Profanity or crude humor
        #[arg(long)]
        profanity: Option<String>,
        /// Medical/treatment information
        #[arg(long)]
        medical: Option<String>,
        /// Contests
        #[arg(long)]
        contests: Option<String>,
        /// Unrestricted web access
        #[arg(long)]
        unrestricted_web: Option<bool>,
    },
}

pub async fn handle(
    cmd: &AgeRatingCommand,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AgeRatingCommand::Get { app_id } => {
            let app_info: Value = client
                .get::<Value>(&format!("/apps/{app_id}/appInfos"), &[("limit", "1")])
                .await?;
            let app_info_id = app_info["data"][0]["id"]
                .as_str()
                .ok_or("no app info found")?;
            client
                .get::<Value>(
                    &format!("/appInfos/{app_info_id}/ageRatingDeclaration"),
                    &[],
                )
                .await
        }
        AgeRatingCommand::Update {
            declaration_id,
            alcohol_tobacco_drugs,
            cartoon_violence,
            realistic_violence,
            sexual_content,
            horror,
            mature_themes,
            gambling,
            profanity,
            medical,
            contests,
            unrestricted_web,
        } => {
            let mut attrs = json!({});
            if let Some(v) = alcohol_tobacco_drugs {
                attrs["alcoholTobaccoOrDrugUseOrReferences"] = json!(v);
            }
            if let Some(v) = cartoon_violence {
                attrs["violenceCartoonOrFantasy"] = json!(v);
            }
            if let Some(v) = realistic_violence {
                attrs["violenceRealistic"] = json!(v);
            }
            if let Some(v) = sexual_content {
                attrs["sexualContentOrNudity"] = json!(v);
            }
            if let Some(v) = horror {
                attrs["horrorOrFearThemes"] = json!(v);
            }
            if let Some(v) = mature_themes {
                attrs["matureOrSuggestiveThemes"] = json!(v);
            }
            if let Some(v) = gambling {
                attrs["gamblingSimulated"] = json!(v);
            }
            if let Some(v) = profanity {
                attrs["profanityOrCrudeHumor"] = json!(v);
            }
            if let Some(v) = medical {
                attrs["medicalOrTreatmentInformation"] = json!(v);
            }
            if let Some(v) = contests {
                attrs["contests"] = json!(v);
            }
            if let Some(v) = unrestricted_web {
                attrs["unrestrictedWebAccess"] = json!(v);
            }
            let body = json!({
                "data": {
                    "type": "ageRatingDeclarations",
                    "id": declaration_id,
                    "attributes": attrs
                }
            });
            client
                .patch(&format!("/ageRatingDeclarations/{declaration_id}"), &body)
                .await
        }
    }
}
