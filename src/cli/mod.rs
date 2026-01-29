pub mod apple;
pub mod google;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "storeops",
    version,
    about = "Manage App Store Connect & Google Play Store"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Output format
    #[arg(long, global = true, default_value = "json")]
    pub output: OutputFormat,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Auth profile to use
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Pagination limit
    #[arg(long, global = true)]
    pub limit: Option<u32>,

    /// Pagination cursor
    #[arg(long, global = true)]
    pub next: Option<String>,

    /// Auto-fetch all pages
    #[arg(long, global = true)]
    pub paginate: bool,

    /// Request timeout in seconds
    #[arg(long, global = true, default_value = "30")]
    pub timeout: u64,

    /// Enable verbose/debug logging
    #[arg(long, global = true)]
    pub verbose: bool,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Markdown,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage authentication and profiles
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Apple App Store Connect commands
    Apple {
        #[command(subcommand)]
        command: apple::AppleCommand,
    },
    /// Google Play Store commands
    Google {
        #[command(subcommand)]
        command: google::GoogleCommand,
    },
    /// Update storeops to the latest release
    Update,
}

#[derive(Subcommand)]
pub enum AuthCommand {
    /// Set up credentials for a store
    Login {
        /// Store to authenticate with
        #[arg(long)]
        store: StoreArg,
        /// Apple: API key ID
        #[arg(long)]
        key_id: Option<String>,
        /// Apple: Issuer ID
        #[arg(long)]
        issuer_id: Option<String>,
        /// Apple: Path to .p8 key file
        #[arg(long)]
        key_path: Option<String>,
        /// Google: Path to service account JSON
        #[arg(long)]
        service_account: Option<String>,
        /// Profile name to save as
        #[arg(long)]
        name: Option<String>,
    },
    /// Switch active profile
    Switch {
        /// Profile name to activate
        profile: String,
    },
    /// Show current auth status
    Status,
    /// Generate config template
    Init,
}

#[derive(Clone, ValueEnum)]
pub enum StoreArg {
    Apple,
    Google,
}
