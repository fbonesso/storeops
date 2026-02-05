pub mod apps;
pub mod availability;
pub mod builds;
pub mod images;
pub mod inapp;
pub mod listings;
pub mod reviews;
pub mod submit;
pub mod sync;
pub mod testers;
pub mod tracks;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum GoogleCommand {
    /// Manage apps
    Apps {
        #[command(subcommand)]
        command: apps::AppsCommand,
    },
    /// Release tracks management
    Tracks {
        #[command(subcommand)]
        command: tracks::TracksCommand,
    },
    /// Build/bundle management
    Builds {
        #[command(subcommand)]
        command: builds::BuildsCommand,
    },
    /// Tester management
    Testers {
        #[command(subcommand)]
        command: testers::TestersCommand,
    },
    /// Submit/promote to production
    Submit {
        /// Package name
        package_name: String,
        /// Target track
        #[arg(long, default_value = "production")]
        track: String,
    },
    /// Ratings and reviews
    Reviews {
        #[command(subcommand)]
        command: reviews::ReviewsCommand,
    },
    /// Manage store listings (title, description, etc. per locale)
    Listings {
        #[command(subcommand)]
        command: listings::ListingsCommand,
    },
    /// Manage images (screenshots, feature graphic, icon)
    Images {
        #[command(subcommand)]
        command: images::ImagesCommand,
    },
    /// Manage in-app products and subscriptions
    Inapp {
        #[command(subcommand)]
        command: inapp::InAppCommand,
    },
    /// Manage country availability
    Availability {
        #[command(subcommand)]
        command: availability::AvailabilityCommand,
    },
    /// Sync metadata and screenshots (bulk pull/push)
    Sync {
        #[command(subcommand)]
        command: sync::SyncCommand,
    },
}

pub async fn execute(
    cmd: &GoogleCommand,
    cli: &crate::cli::Cli,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let config = crate::config::Config::load()?;
    let sa_path = crate::auth::store::resolve_google_credentials(&config, cli.profile.as_deref())?;
    let token = crate::auth::google::get_access_token(&sa_path).await?;
    let client = crate::api::google_client::GoogleClient::new(token);

    match cmd {
        GoogleCommand::Apps { command } => apps::handle(command, &client).await,
        GoogleCommand::Tracks { command } => tracks::handle(command, &client).await,
        GoogleCommand::Builds { command } => builds::handle(command, &client).await,
        GoogleCommand::Testers { command } => testers::handle(command, &client).await,
        GoogleCommand::Submit {
            package_name,
            track,
        } => submit::handle(package_name, track, &client).await,
        GoogleCommand::Reviews { command } => reviews::handle(command, &client).await,
        GoogleCommand::Listings { command } => listings::handle(command, &client).await,
        GoogleCommand::Images { command } => images::handle(command, &client).await,
        GoogleCommand::Inapp { command } => inapp::handle(command, &client).await,
        GoogleCommand::Availability { command } => availability::handle(command, &client).await,
        GoogleCommand::Sync { command } => sync::handle(command, &client).await,
    }
}
