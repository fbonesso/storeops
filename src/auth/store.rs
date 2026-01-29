use crate::config::Config;
use crate::config::profiles::Credentials;

pub fn resolve_apple_credentials(
    config: &Config,
    profile_name: Option<&str>,
) -> Result<(String, String, Vec<u8>), String> {
    if let (Ok(key_id), Ok(issuer_id), Ok(key_path)) = (
        std::env::var("STOREOPS_APPLE_KEY_ID"),
        std::env::var("STOREOPS_APPLE_ISSUER_ID"),
        std::env::var("STOREOPS_APPLE_KEY_PATH"),
    ) {
        let key = std::fs::read(&key_path).map_err(|e| format!("cannot read key: {e}"))?;
        return Ok((key_id, issuer_id, key));
    }

    let profile = match profile_name {
        Some(name) => config
            .profiles
            .get(name)
            .ok_or_else(|| format!("profile '{name}' not found"))?,
        None => config.active_profile().ok_or("no active profile set")?,
    };

    match &profile.credentials {
        Credentials::Apple {
            key_id,
            issuer_id,
            key_path,
        } => {
            let key = std::fs::read(key_path).map_err(|e| format!("cannot read key: {e}"))?;
            Ok((key_id.clone(), issuer_id.clone(), key))
        }
        _ => Err("active profile is not an Apple profile".to_string()),
    }
}

pub fn resolve_google_credentials(
    config: &Config,
    profile_name: Option<&str>,
) -> Result<String, String> {
    if let Ok(path) = std::env::var("STOREOPS_GOOGLE_SERVICE_ACCOUNT") {
        return Ok(path);
    }

    let profile = match profile_name {
        Some(name) => config
            .profiles
            .get(name)
            .ok_or_else(|| format!("profile '{name}' not found"))?,
        None => config.active_profile().ok_or("no active profile set")?,
    };

    match &profile.credentials {
        Credentials::Google {
            service_account_path,
        } => Ok(service_account_path.clone()),
        _ => Err("active profile is not a Google profile".to_string()),
    }
}
