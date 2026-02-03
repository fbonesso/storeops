mod api;
mod auth;
mod cli;
mod config;
mod output;
mod repl;
mod update;

use clap::Parser;
use cli::{AuthCommand, Cli, Command};
use config::profiles::{Credentials, Profile, Store};
use config::Config;
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

    let json_output = cli.json;
    let pretty = cli.pretty;

    let is_update = matches!(cli.command, Some(Command::Update));
    if !is_update {
        tokio::spawn(update::check_for_update_background());
    }

    let result = run(cli).await;

    match result {
        Ok(value) => {
            println!("{}", output::render_value(&value, json_output, pretty));
            process::exit(0);
        }
        Err(e) => {
            let err = json!({ "error": e.to_string() });
            eprintln!(
                "{}",
                serde_json::to_string(&err).unwrap_or_else(|_| format!("{{\"error\":\"{}\"}}", e))
            );
            process::exit(1);
        }
    }
}

pub async fn run(cli: Cli) -> Result<Value, Box<dyn std::error::Error>> {
    match &cli.command {
        Some(Command::Auth { command }) => handle_auth(command).await,
        Some(Command::Apple { command }) => cli::apple::execute(command, &cli).await,
        Some(Command::Google { command }) => cli::google::execute(command, &cli).await,
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
            let path = Config::config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            Ok(json!({
                "status": "ok",
                "config_path": path,
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
