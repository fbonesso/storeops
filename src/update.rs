use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const REPO: &str = "fbonesso/storeops";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL_SECS: u64 = 86400; // 24 hours

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

fn version_file_path() -> Option<PathBuf> {
    crate::config::Config::config_dir().map(|d| d.join(".last_version_check"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn should_check() -> bool {
    let path = match version_file_path() {
        Some(p) => p,
        None => return false,
    };
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let last: u64 = content.trim().parse().unwrap_or(0);
            now_secs().saturating_sub(last) >= CHECK_INTERVAL_SECS
        }
        Err(_) => true,
    }
}

fn record_check() {
    if let Some(path) = version_file_path() {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let _ = std::fs::write(path, now_secs().to_string());
    }
}

fn normalize_version(v: &str) -> &str {
    v.strip_prefix('v').unwrap_or(v)
}

fn is_newer(remote: &str, local: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        normalize_version(v)
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let r = parse(remote);
    let l = parse(local);
    r > l
}

async fn fetch_latest_release() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", format!("storeops/{CURRENT_VERSION}"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitHub API error {status}: {body}").into());
    }
    Ok(resp.json().await?)
}

fn detect_target() -> String {
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    let os = if cfg!(target_os = "linux") {
        "unknown-linux-gnu"
    } else if cfg!(target_os = "macos") {
        "apple-darwin"
    } else if cfg!(target_os = "windows") {
        "pc-windows-msvc"
    } else {
        "unknown"
    };

    format!("{arch}-{os}")
}

fn find_asset_for_target<'a>(assets: &'a [GitHubAsset], target: &str) -> Option<&'a GitHubAsset> {
    assets.iter().find(|a| a.name.contains(target))
}

fn current_exe_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let exe = env::current_exe()?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "cannot determine executable directory".into())
}

pub async fn check_for_update_background() {
    if env::var("STOREOPS_NO_UPDATE_CHECK").is_ok() {
        return;
    }
    if !should_check() {
        return;
    }

    record_check();

    let release = match fetch_latest_release().await {
        Ok(r) => r,
        Err(_) => return,
    };

    let remote = normalize_version(&release.tag_name);
    if is_newer(remote, CURRENT_VERSION) {
        eprintln!(
            "A new version of storeops is available: {} -> {} (run `storeops update` to upgrade)",
            CURRENT_VERSION, remote
        );
    }
}

pub async fn handle_update() -> Result<Value, Box<dyn std::error::Error>> {
    eprintln!("Checking for updates...");

    let release = fetch_latest_release().await?;
    let remote = normalize_version(&release.tag_name);

    if !is_newer(remote, CURRENT_VERSION) {
        return Ok(json!({
            "status": "up_to_date",
            "current_version": CURRENT_VERSION,
        }));
    }

    eprintln!("Updating storeops: {} -> {}", CURRENT_VERSION, remote);

    let target = detect_target();
    let asset = find_asset_for_target(&release.assets, &target)
        .ok_or_else(|| format!("no release asset found for target {target}"))?;

    eprintln!("Downloading {}...", asset.name);

    let client = reqwest::Client::new();
    let bytes = client
        .get(&asset.browser_download_url)
        .header("User-Agent", format!("storeops/{CURRENT_VERSION}"))
        .header("Accept", "application/octet-stream")
        .send()
        .await?
        .bytes()
        .await?;

    let tmpdir = std::env::temp_dir().join("storeops-update");
    let _ = std::fs::remove_dir_all(&tmpdir);
    std::fs::create_dir_all(&tmpdir)?;

    let archive_path = tmpdir.join(&asset.name);
    std::fs::write(&archive_path, &bytes)?;

    eprintln!("Extracting...");
    let install_dir = current_exe_dir()?;

    if asset.name.ends_with(".zip") {
        extract_zip(&archive_path, &tmpdir)?;
    } else {
        extract_tar_gz(&archive_path, &tmpdir)?;
    }

    let main_bin = if cfg!(target_os = "windows") {
        "storeops.exe"
    } else {
        "storeops"
    };

    let src = tmpdir.join(main_bin);
    if !src.exists() {
        return Err(format!("{main_bin} not found in archive").into());
    }
    let dest = install_dir.join(main_bin);
    let backup = install_dir.join(format!("{main_bin}.bak"));

    if dest.exists() {
        let _ = std::fs::rename(&dest, &backup);
    }
    match std::fs::copy(&src, &dest) {
        Ok(_) => {
            let _ = std::fs::remove_file(&backup);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
            }
        }
        Err(e) => {
            if backup.exists() {
                let _ = std::fs::rename(&backup, &dest);
            }
            return Err(format!("failed to install {main_bin}: {e}").into());
        }
    }

    let _ = std::fs::remove_dir_all(&tmpdir);
    record_check();

    Ok(json!({
        "status": "updated",
        "previous_version": CURRENT_VERSION,
        "new_version": remote,
    }))
}

fn extract_tar_gz(
    archive: &std::path::Path,
    dest: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(archive)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

fn extract_zip(
    archive: &std::path::Path,
    dest: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    zip.extract(dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_newer_detects_higher_version() {
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("v0.1.1", "0.1.0"));
    }

    #[test]
    fn is_newer_rejects_same_or_lower() {
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("v0.1.0", "0.2.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn normalize_strips_v_prefix() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
        assert_eq!(normalize_version("1.2.3"), "1.2.3");
    }

    #[test]
    fn detect_target_returns_known_triple() {
        let target = detect_target();
        assert!(
            target.contains("linux") || target.contains("darwin") || target.contains("windows"),
            "unexpected target: {target}"
        );
    }
}
