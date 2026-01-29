mod api;
mod auth;
mod cli;
mod config;
mod output;
mod repl;
mod update;

use clap::Parser;
use cli::{AuthCommand, Cli, Command};
use config::Config;
use config::profiles::{Credentials, Profile, Store};
use serde_json::{json, Value};
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.command.is_none() {
        tokio::spawn(update::check_for_update_background());
        repl::run_repl().await;
        process::exit(0);
    }

    let pretty = cli.pretty;

    let is_update = matches!(cli.command, Some(Command::Update));
    if !is_update {
        tokio::spawn(update::check_for_update_background());
    }

    let result = run(cli).await;

    match result {
        Ok(value) => {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
            } else {
                println!("{}", serde_json::to_string(&value).unwrap());
            }
            process::exit(0);
        }
        Err(e) => {
            let err = json!({ "error": e.to_string() });
            eprintln!("{}", serde_json::to_string(&err).unwrap());
            process::exit(1);
        }
    }
}

pub async fn run(cli: Cli) -> Result<Value, Box<dyn std::error::Error>> {
    match &cli.command {
        Some(Command::Auth { command }) => handle_auth(command).await,
        Some(Command::Apple { command }) => handle_apple(command, &cli).await,
        Some(Command::Google { command }) => handle_google(command, &cli).await,
        Some(Command::Update) => update::handle_update().await,
        None => Err("no command provided".into()),
    }
}

async fn handle_auth(cmd: &AuthCommand) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        AuthCommand::Init => {
            let mut config = Config::load().unwrap_or_default();
            if config.profiles.is_empty() {
                config.profiles.insert(
                    "apple-default".to_string(),
                    Profile {
                        store: Store::Apple,
                        credentials: Credentials::Apple {
                            key_id: "YOUR_KEY_ID".to_string(),
                            issuer_id: "YOUR_ISSUER_ID".to_string(),
                            key_path: "/path/to/AuthKey.p8".to_string(),
                        },
                    },
                );
                config.profiles.insert(
                    "google-default".to_string(),
                    Profile {
                        store: Store::Google,
                        credentials: Credentials::Google {
                            service_account_path: "/path/to/service-account.json".to_string(),
                        },
                    },
                );
            }
            config.save()?;
            let path = Config::config_path().unwrap();
            Ok(json!({
                "status": "ok",
                "config_path": path.display().to_string(),
                "message": "Config template created. Edit the file to add your credentials."
            }))
        }
        AuthCommand::Status => {
            let config = Config::load()?;
            let profiles: Vec<Value> = config
                .profiles
                .iter()
                .map(|(name, p)| {
                    json!({
                        "name": name,
                        "store": p.store.to_string(),
                        "active": config.active_profile.as_deref() == Some(name.as_str()),
                    })
                })
                .collect();
            Ok(json!({
                "active_profile": config.active_profile,
                "profiles": profiles,
            }))
        }
        AuthCommand::Switch { profile } => {
            let mut config = Config::load()?;
            if !config.profiles.contains_key(profile) {
                return Err(format!("profile '{}' not found", profile).into());
            }
            config.active_profile = Some(profile.clone());
            config.save()?;
            Ok(json!({
                "status": "ok",
                "active_profile": profile,
            }))
        }
        AuthCommand::Login {
            store,
            key_id,
            issuer_id,
            key_path,
            service_account,
            name,
        } => {
            let mut config = Config::load().unwrap_or_default();
            let profile_name;
            let profile;

            match store {
                cli::StoreArg::Apple => {
                    let kid = key_id.as_ref().ok_or("--key-id required for Apple")?;
                    let iss = issuer_id.as_ref().ok_or("--issuer-id required for Apple")?;
                    let kp = key_path.as_ref().ok_or("--key-path required for Apple")?;
                    profile_name = name.clone().unwrap_or_else(|| "apple-default".to_string());
                    profile = Profile {
                        store: Store::Apple,
                        credentials: Credentials::Apple {
                            key_id: kid.clone(),
                            issuer_id: iss.clone(),
                            key_path: kp.clone(),
                        },
                    };
                }
                cli::StoreArg::Google => {
                    let sa = service_account
                        .as_ref()
                        .ok_or("--service-account required for Google")?;
                    profile_name = name.clone().unwrap_or_else(|| "google-default".to_string());
                    profile = Profile {
                        store: Store::Google,
                        credentials: Credentials::Google {
                            service_account_path: sa.clone(),
                        },
                    };
                }
            }

            config.profiles.insert(profile_name.clone(), profile);
            if config.active_profile.is_none() {
                config.active_profile = Some(profile_name.clone());
            }
            config.save()?;
            Ok(json!({
                "status": "ok",
                "profile": profile_name,
                "message": "Credentials saved.",
            }))
        }
    }
}

async fn handle_apple(
    cmd: &cli::apple::AppleCommand,
    cli: &Cli,
) -> Result<Value, Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let (key_id, issuer_id, key_pem) =
        auth::store::resolve_apple_credentials(&config, cli.profile.as_deref())?;
    let token = auth::apple::generate_token(&key_id, &issuer_id, &key_pem)?;
    let client = api::apple_client::AppleClient::new(token);

    match cmd {
        cli::apple::AppleCommand::Apps { command } => {
            cli::apple::apps::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Versions { command } => {
            cli::apple::versions::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Builds { command } => {
            cli::apple::builds::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Testflight { command } => {
            cli::apple::testflight::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Submit { app_id, version } => {
            cli::apple::submit::handle(app_id, version, &client).await
        }
        cli::apple::AppleCommand::Reviews { command } => {
            cli::apple::reviews::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Devices { command } => {
            cli::apple::devices::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Analytics { command } => {
            cli::apple::analytics::handle(command, &client).await
        }
        cli::apple::AppleCommand::Metadata { command } => {
            cli::apple::metadata::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Screenshots { command } => {
            cli::apple::screenshots::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Previews { command } => {
            cli::apple::previews::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Pricing { command } => {
            cli::apple::pricing::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::AgeRating { command } => {
            cli::apple::age_rating::handle(command, &client).await
        }
        cli::apple::AppleCommand::PhasedRelease { command } => {
            cli::apple::phased_release::handle(command, &client).await
        }
        cli::apple::AppleCommand::Iap { command } => {
            cli::apple::iap::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Subscriptions { command } => {
            cli::apple::subscriptions::handle(command, &client, cli.limit).await
        }
        cli::apple::AppleCommand::Availability { command } => {
            cli::apple::availability::handle(command, &client, cli.limit).await
        }
    }
}

async fn handle_google(
    cmd: &cli::google::GoogleCommand,
    cli: &Cli,
) -> Result<Value, Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let sa_path = auth::store::resolve_google_credentials(&config, cli.profile.as_deref())?;
    let token = auth::google::get_access_token(&sa_path).await?;
    let client = api::google_client::GoogleClient::new(token);

    match cmd {
        cli::google::GoogleCommand::Apps { command } => {
            cli::google::apps::handle(command, &client).await
        }
        cli::google::GoogleCommand::Tracks { command } => {
            cli::google::tracks::handle(command, &client).await
        }
        cli::google::GoogleCommand::Builds { command } => {
            cli::google::builds::handle(command, &client).await
        }
        cli::google::GoogleCommand::Testers { command } => {
            cli::google::testers::handle(command, &client).await
        }
        cli::google::GoogleCommand::Submit {
            package_name,
            track,
        } => cli::google::submit::handle(package_name, track, &client).await,
        cli::google::GoogleCommand::Reviews { command } => {
            cli::google::reviews::handle(command, &client).await
        }
        cli::google::GoogleCommand::Reports { command } => {
            cli::google::reports::handle(command, &client).await
        }
        cli::google::GoogleCommand::Listings { command } => {
            cli::google::listings::handle(command, &client).await
        }
        cli::google::GoogleCommand::Images { command } => {
            cli::google::images::handle(command, &client).await
        }
        cli::google::GoogleCommand::Inapp { command } => {
            cli::google::inapp::handle(command, &client).await
        }
        cli::google::GoogleCommand::Availability { command } => {
            cli::google::availability::handle(command, &client).await
        }
    }
}
