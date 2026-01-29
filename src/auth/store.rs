use crate::config::profiles::Credentials;
use crate::config::Config;
use std::path::Path;

fn validate_file_path(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("file not found: {path}"));
    }
    let canonical = p
        .canonicalize()
        .map_err(|e| format!("cannot resolve path {path}: {e}"))?;
    if !canonical.is_file() {
        return Err(format!("not a regular file: {path}"));
    }
    Ok(canonical.to_string_lossy().into_owned())
}

pub fn resolve_apple_credentials(
    config: &Config,
    profile_name: Option<&str>,
) -> Result<(String, String, Vec<u8>), String> {
    if let (Ok(key_id), Ok(issuer_id), Ok(key_path)) = (
        std::env::var("STOREOPS_APPLE_KEY_ID"),
        std::env::var("STOREOPS_APPLE_ISSUER_ID"),
        std::env::var("STOREOPS_APPLE_KEY_PATH"),
    ) {
        let resolved = validate_file_path(&key_path)?;
        let key = std::fs::read(&resolved).map_err(|e| format!("cannot read key: {e}"))?;
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
            let resolved = validate_file_path(key_path)?;
            let key = std::fs::read(&resolved).map_err(|e| format!("cannot read key: {e}"))?;
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
        return validate_file_path(&path);
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
        } => validate_file_path(service_account_path),
        _ => Err("active profile is not a Google profile".to_string()),
    }
}
