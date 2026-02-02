pub mod age_rating;
pub mod analytics;
pub mod apps;
pub mod availability;
pub mod builds;
pub mod devices;
pub mod iap;
pub mod metadata;
pub mod phased_release;
pub mod previews;
pub mod pricing;
pub mod reviews;
pub mod screenshots;
pub mod submit;
pub mod subscriptions;
pub mod testflight;
pub mod versions;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum AppleCommand {
    /// Manage apps
    Apps {
        #[command(subcommand)]
        command: apps::AppsCommand,
    },
    /// Manage app versions
    Versions {
        #[command(subcommand)]
        command: versions::VersionsCommand,
    },
    /// Manage builds
    Builds {
        #[command(subcommand)]
        command: builds::BuildsCommand,
    },
    /// TestFlight management
    Testflight {
        #[command(subcommand)]
        command: testflight::TestflightCommand,
    },
    /// Submit for review
    Submit {
        /// App ID
        app_id: String,
        /// Version string
        #[arg(long)]
        version: String,
    },
    /// Customer reviews
    Reviews {
        #[command(subcommand)]
        command: reviews::ReviewsCommand,
    },
    /// Device management
    Devices {
        #[command(subcommand)]
        command: devices::DevicesCommand,
    },
    /// Sales and finance analytics
    Analytics {
        #[command(subcommand)]
        command: analytics::AnalyticsCommand,
    },
    /// Manage metadata, localizations, categories
    Metadata {
        #[command(subcommand)]
        command: metadata::MetadataCommand,
    },
    /// Manage screenshots
    Screenshots {
        #[command(subcommand)]
        command: screenshots::ScreenshotsCommand,
    },
    /// Manage app preview videos
    Previews {
        #[command(subcommand)]
        command: previews::PreviewsCommand,
    },
    /// Manage pricing and price schedules
    Pricing {
        #[command(subcommand)]
        command: pricing::PricingCommand,
    },
    /// Manage age rating declarations
    AgeRating {
        #[command(subcommand)]
        command: age_rating::AgeRatingCommand,
    },
    /// Manage phased releases
    PhasedRelease {
        #[command(subcommand)]
        command: phased_release::PhasedReleaseCommand,
    },
    /// Manage in-app purchases
    Iap {
        #[command(subcommand)]
        command: iap::IapCommand,
    },
    /// Manage auto-renewable subscriptions
    Subscriptions {
        #[command(subcommand)]
        command: subscriptions::SubscriptionsCommand,
    },
    /// Manage territory availability
    Availability {
        #[command(subcommand)]
        command: availability::AvailabilityCommand,
    },
}

pub async fn execute(
    cmd: &AppleCommand,
    cli: &crate::cli::Cli,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let config = crate::config::Config::load()?;
    let (key_id, issuer_id, key_pem) =
        crate::auth::store::resolve_apple_credentials(&config, cli.profile.as_deref())?;
    let token = crate::auth::apple::generate_token(&key_id, &issuer_id, &key_pem)?;
    let client = crate::api::apple_client::AppleClient::new(token);

    match cmd {
        AppleCommand::Apps { command } => apps::handle(command, &client, cli.limit).await,
        AppleCommand::Versions { command } => versions::handle(command, &client, cli.limit).await,
        AppleCommand::Builds { command } => builds::handle(command, &client, cli.limit).await,
        AppleCommand::Testflight { command } => {
            testflight::handle(command, &client, cli.limit).await
        }
        AppleCommand::Submit { app_id, version } => submit::handle(app_id, version, &client).await,
        AppleCommand::Reviews { command } => reviews::handle(command, &client, cli.limit).await,
        AppleCommand::Devices { command } => devices::handle(command, &client, cli.limit).await,
        AppleCommand::Analytics { command } => analytics::handle(command, &client).await,
        AppleCommand::Metadata { command } => metadata::handle(command, &client, cli.limit).await,
        AppleCommand::Screenshots { command } => {
            screenshots::handle(command, &client, cli.limit).await
        }
        AppleCommand::Previews { command } => previews::handle(command, &client, cli.limit).await,
        AppleCommand::Pricing { command } => pricing::handle(command, &client, cli.limit).await,
        AppleCommand::AgeRating { command } => age_rating::handle(command, &client).await,
        AppleCommand::PhasedRelease { command } => phased_release::handle(command, &client).await,
        AppleCommand::Iap { command } => iap::handle(command, &client, cli.limit).await,
        AppleCommand::Subscriptions { command } => {
            subscriptions::handle(command, &client, cli.limit).await
        }
        AppleCommand::Availability { command } => {
            availability::handle(command, &client, cli.limit).await
        }
    }
}
