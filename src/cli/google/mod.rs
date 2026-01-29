pub mod apps;
pub mod availability;
pub mod builds;
pub mod images;
pub mod inapp;
pub mod listings;
pub mod reviews;
pub mod submit;
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
}
