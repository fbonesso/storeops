//! Apple App Store Connect sync commands for batch metadata and screenshot operations.
//!
//! Provides pull (download) and push (upload) functionality for app metadata and screenshots
//! across all locales in a single operation.

use clap::Subcommand;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

use crate::api::apple_client::AppleClient;

/// App Store Connect locale codes mapped to internal standardized codes.
/// Format: "asc_locale" -> "internal_locale"
fn asc_to_internal_locale(asc_locale: &str) -> String {
    match asc_locale {
        "ja" => "ja-JP".to_string(),
        "ko" => "ko-KR".to_string(),
        "it" => "it-IT".to_string(),
        "ru" => "ru-RU".to_string(),
        "tr" => "tr-TR".to_string(),
        "zh-Hans" => "zh-CN".to_string(),
        "zh-Hant" => "zh-TW".to_string(),
        "ar-SA" => "ar".to_string(),
        "hi" => "hi-IN".to_string(),
        _ => asc_locale.to_string(),
    }
}

/// Internal locale codes mapped to App Store Connect codes.
/// Format: "internal_locale" -> "asc_locale"
fn internal_to_asc_locale(internal_locale: &str) -> String {
    match internal_locale {
        "ja-JP" => "ja".to_string(),
        "ko-KR" => "ko".to_string(),
        "it-IT" => "it".to_string(),
        "ru-RU" => "ru".to_string(),
        "tr-TR" => "tr".to_string(),
        "zh-CN" => "zh-Hans".to_string(),
        "zh-TW" => "zh-Hant".to_string(),
        "ar" => "ar-SA".to_string(),
        "hi-IN" => "hi".to_string(),
        _ => internal_locale.to_string(),
    }
}

/// Screenshot display type to directory name mapping
fn display_type_to_dir(display_type: &str) -> &str {
    match display_type {
        // APP_IPHONE_67 covers both 6.7" (1290x2796) and 6.9" (1320x2868) devices
        "APP_IPHONE_67" => "iphone67",
        "APP_IPHONE_65" => "iphone65",
        "APP_IPHONE_61" => "iphone61",
        "APP_IPHONE_58" => "iphone58",
        "APP_IPHONE_55" => "iphone55",
        "APP_IPAD_PRO_129" | "APP_IPAD_PRO_3GEN_129" => "ipadPro129",
        "APP_IPAD_PRO_11" | "APP_IPAD_PRO_3GEN_11" => "ipadPro11",
        _ => display_type,
    }
}

/// Directory name to screenshot display type mapping
fn dir_to_display_type(dir_name: &str) -> &str {
    match dir_name {
        // iphone69 uses APP_IPHONE_67 - Apple uses this type for both 6.7" and 6.9" displays
        // iPhone 16 Pro Max (6.9" - 1320x2868) and iPhone 14 Pro Max (6.7" - 1290x2796)
        // both use the APP_IPHONE_67 display type
        "iphone69" | "iphone67" => "APP_IPHONE_67",
        "iphone65" => "APP_IPHONE_65",
        "iphone61" => "APP_IPHONE_61",
        "iphone58" => "APP_IPHONE_58",
        "iphone55" => "APP_IPHONE_55",
        "ipadPro129" => "APP_IPAD_PRO_3GEN_129",
        "ipadPro11" => "APP_IPAD_PRO_3GEN_11",
        _ => dir_name,
    }
}

#[derive(Subcommand)]
pub enum SyncCommand {
    /// Pull (download) all metadata and screenshots for an app
    Pull {
        /// iOS Bundle ID (e.g., com.example.app)
        bundle_id: String,
        /// Output directory for downloaded metadata
        #[arg(long)]
        output_dir: PathBuf,
        /// Skip downloading screenshots
        #[arg(long, default_value = "false")]
        skip_screenshots: bool,
        /// Skip downloading metadata
        #[arg(long, default_value = "false")]
        skip_metadata: bool,
        /// Return screenshot URLs in JSON output instead of downloading
        #[arg(long, default_value = "false")]
        urls_only: bool,
    },
    /// Push (upload) all metadata and screenshots for an app
    Push {
        /// iOS Bundle ID (e.g., com.example.app)
        bundle_id: String,
        /// Directory containing metadata to upload
        #[arg(long)]
        metadata_dir: PathBuf,
        /// Skip uploading screenshots
        #[arg(long, default_value = "false")]
        skip_screenshots: bool,
        /// Skip uploading metadata
        #[arg(long, default_value = "false")]
        skip_metadata: bool,
    },
}

pub async fn handle(
    cmd: &SyncCommand,
    client: &AppleClient,
    _limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        SyncCommand::Pull {
            bundle_id,
            output_dir,
            skip_screenshots,
            skip_metadata,
            urls_only,
        } => {
            handle_pull(
                bundle_id,
                output_dir,
                *skip_screenshots,
                *skip_metadata,
                *urls_only,
                client,
            )
            .await
        }
        SyncCommand::Push {
            bundle_id,
            metadata_dir,
            skip_screenshots,
            skip_metadata,
        } => {
            handle_push(
                bundle_id,
                metadata_dir,
                *skip_screenshots,
                *skip_metadata,
                client,
            )
            .await
        }
    }
}

/// Lookup app by bundle ID and return the app ID
async fn lookup_app_by_bundle_id(
    bundle_id: &str,
    client: &AppleClient,
) -> Result<String, Box<dyn std::error::Error>> {
    let apps: Value = client
        .get("/apps", &[("filter[bundleId]", bundle_id), ("limit", "1")])
        .await?;

    let app_id = apps["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|app| app["id"].as_str())
        .ok_or_else(|| format!("App not found for bundle ID: {}", bundle_id))?;

    Ok(app_id.to_string())
}

/// States where metadata can be edited
const EDITABLE_STATES: &[&str] = &["PREPARE_FOR_SUBMISSION", "DEVELOPER_REJECTED", "REJECTED"];

/// Get the editable (or latest) App Store version for an app
async fn get_editable_version(
    app_id: &str,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    // First try to get editable version (non-live states)
    let versions: Value = client
        .get(
            &format!("/apps/{app_id}/appStoreVersions"),
            &[
                ("filter[appStoreState]", "PREPARE_FOR_SUBMISSION,READY_FOR_REVIEW,WAITING_FOR_REVIEW,IN_REVIEW,PENDING_DEVELOPER_RELEASE,PENDING_APPLE_RELEASE"),
                ("limit", "1"),
            ],
        )
        .await?;

    if let Some(version) = versions["data"].as_array().and_then(|arr| arr.first()) {
        return Ok(version.clone());
    }

    // Fall back to any version
    let versions: Value = client
        .get(
            &format!("/apps/{app_id}/appStoreVersions"),
            &[("limit", "1")],
        )
        .await?;

    versions["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .cloned()
        .ok_or_else(|| "No App Store version found".into())
}

/// Get or create an editable version for pushing metadata
async fn get_or_create_editable_version(
    app_id: &str,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    // First check for existing editable versions
    let versions: Value = client
        .get(
            &format!("/apps/{app_id}/appStoreVersions"),
            &[
                ("filter[appStoreState]", &EDITABLE_STATES.join(",")),
                ("limit", "1"),
            ],
        )
        .await?;

    if let Some(version) = versions["data"].as_array().and_then(|arr| arr.first()) {
        let state = version["attributes"]["appStoreState"]
            .as_str()
            .unwrap_or("");
        eprintln!("Found editable version in state: {}", state);
        return Ok(version.clone());
    }

    // Get the latest version to determine the version string for the new version
    let latest_versions: Value = client
        .get(
            &format!("/apps/{app_id}/appStoreVersions"),
            &[("limit", "1")],
        )
        .await?;

    let latest_version = latest_versions["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .ok_or("No existing version found")?;

    let current_state = latest_version["attributes"]["appStoreState"]
        .as_str()
        .unwrap_or("");
    let current_version_string = latest_version["attributes"]["versionString"]
        .as_str()
        .unwrap_or("1.0.0");

    eprintln!(
        "Current version {} is in state: {} - creating new version",
        current_version_string, current_state
    );

    // Increment the version string (simple approach: increment patch version)
    let new_version_string = increment_version(current_version_string);
    eprintln!("Creating new version: {}", new_version_string);

    // Create a new version
    let body = json!({
        "data": {
            "type": "appStoreVersions",
            "attributes": {
                "versionString": new_version_string,
                "platform": "IOS"
            },
            "relationships": {
                "app": {
                    "data": {
                        "type": "apps",
                        "id": app_id
                    }
                }
            }
        }
    });

    let result: Value = client.post("/appStoreVersions", &body).await?;

    result["data"]
        .as_object()
        .map(|_| result["data"].clone())
        .ok_or_else(|| "Failed to create new version".into())
}

/// Increment a version string (e.g., "1.2.3" -> "1.2.4")
fn increment_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    match parts.len() {
        3 => {
            let major = parts[0];
            let minor = parts[1];
            let patch: u32 = parts[2].parse().unwrap_or(0);
            format!("{}.{}.{}", major, minor, patch + 1)
        }
        2 => {
            let major = parts[0];
            let minor: u32 = parts[1].parse().unwrap_or(0);
            format!("{}.{}", major, minor + 1)
        }
        _ => format!("{}.1", version),
    }
}

/// States where app info can be edited
const APP_INFO_EDITABLE_STATES: &[&str] = &["READY_FOR_SUBMISSION", "PREPARE_FOR_SUBMISSION"];

/// Get the latest app info for an app and check if it's editable
async fn get_app_info(
    app_id: &str,
    client: &AppleClient,
) -> Result<(Value, bool), Box<dyn std::error::Error>> {
    let infos: Value = client
        .get(&format!("/apps/{app_id}/appInfos"), &[("limit", "5")])
        .await?;

    // Find the first editable app info, or fall back to the first one
    if let Some(arr) = infos["data"].as_array() {
        // First try to find an editable one
        for info in arr {
            let state = info["attributes"]["appStoreState"].as_str().unwrap_or("");
            if APP_INFO_EDITABLE_STATES.contains(&state) {
                return Ok((info.clone(), true));
            }
        }
        // Fall back to first one but mark as non-editable
        if let Some(first) = arr.first() {
            let state = first["attributes"]["appStoreState"].as_str().unwrap_or("");
            eprintln!(
                "App info state: {} (may not be editable for name/subtitle)",
                state
            );
            return Ok((first.clone(), false));
        }
    }

    Err("No app info found".into())
}

async fn handle_pull(
    bundle_id: &str,
    output_dir: &PathBuf,
    skip_screenshots: bool,
    skip_metadata: bool,
    urls_only: bool,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    eprintln!("Looking up app: {}", bundle_id);
    let app_id = lookup_app_by_bundle_id(bundle_id, client).await?;
    eprintln!("Found app ID: {}", app_id);

    let version = get_editable_version(&app_id, client).await?;
    let version_id = version["id"].as_str().ok_or("Version ID not found")?;
    eprintln!("Using version: {}", version_id);

    let (app_info, _) = get_app_info(&app_id, client).await?;
    let app_info_id = app_info["id"].as_str().ok_or("App Info ID not found")?;
    eprintln!("App info ID: {}", app_info_id);

    // Create output directory
    fs::create_dir_all(output_dir).await?;

    let mut locales_downloaded = Vec::new();
    let mut screenshots_downloaded = 0u32;
    // For urls_only mode: locale -> device -> [urls]
    let mut screenshot_urls: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    if !skip_metadata {
        // Get version localizations (description, keywords, whatsNew, etc.)
        eprintln!("Fetching version localizations...");
        let version_locs: Value = client
            .get(
                &format!("/appStoreVersions/{version_id}/appStoreVersionLocalizations"),
                &[("limit", "100")],
            )
            .await?;

        // Get app info localizations (name, subtitle)
        eprintln!("Fetching app info localizations...");
        let app_info_locs: Value = client
            .get(
                &format!("/appInfos/{app_info_id}/appInfoLocalizations"),
                &[("limit", "100")],
            )
            .await?;

        // Build a map of app info localizations by locale
        let mut app_info_by_locale: HashMap<String, &Value> = HashMap::new();
        if let Some(locs) = app_info_locs["data"].as_array() {
            for loc in locs {
                if let Some(locale) = loc["attributes"]["locale"].as_str() {
                    app_info_by_locale.insert(locale.to_string(), loc);
                }
            }
        }

        // Process version localizations
        if let Some(locs) = version_locs["data"].as_array() {
            for loc in locs {
                let asc_locale = loc["attributes"]["locale"].as_str().unwrap_or("en-US");
                let internal_locale = asc_to_internal_locale(asc_locale);
                let locale_dir = output_dir.join(&internal_locale);
                fs::create_dir_all(&locale_dir).await?;

                // Version-level metadata
                let attrs = &loc["attributes"];

                // description.txt
                if let Some(desc) = attrs["description"].as_str() {
                    if !desc.is_empty() {
                        fs::write(locale_dir.join("description.txt"), desc).await?;
                    }
                }

                // keywords.txt
                if let Some(keywords) = attrs["keywords"].as_str() {
                    if !keywords.is_empty() {
                        fs::write(locale_dir.join("keywords.txt"), keywords).await?;
                    }
                }

                // release_notes.txt (whatsNew)
                if let Some(whats_new) = attrs["whatsNew"].as_str() {
                    if !whats_new.is_empty() {
                        fs::write(locale_dir.join("release_notes.txt"), whats_new).await?;
                    }
                }

                // promotional_text.txt
                if let Some(promo) = attrs["promotionalText"].as_str() {
                    if !promo.is_empty() {
                        fs::write(locale_dir.join("promotional_text.txt"), promo).await?;
                    }
                }

                // marketing_url.txt
                if let Some(url) = attrs["marketingUrl"].as_str() {
                    if !url.is_empty() {
                        fs::write(locale_dir.join("marketing_url.txt"), url).await?;
                    }
                }

                // support_url.txt
                if let Some(url) = attrs["supportUrl"].as_str() {
                    if !url.is_empty() {
                        fs::write(locale_dir.join("support_url.txt"), url).await?;
                    }
                }

                // App info-level metadata (name, subtitle)
                if let Some(app_info_loc) = app_info_by_locale.get(asc_locale) {
                    let app_attrs = &app_info_loc["attributes"];

                    // name.txt
                    if let Some(name) = app_attrs["name"].as_str() {
                        if !name.is_empty() {
                            fs::write(locale_dir.join("name.txt"), name).await?;
                        }
                    }

                    // subtitle.txt
                    if let Some(subtitle) = app_attrs["subtitle"].as_str() {
                        if !subtitle.is_empty() {
                            fs::write(locale_dir.join("subtitle.txt"), subtitle).await?;
                        }
                    }

                    // privacy_url.txt
                    if let Some(url) = app_attrs["privacyPolicyUrl"].as_str() {
                        if !url.is_empty() {
                            fs::write(locale_dir.join("privacy_url.txt"), url).await?;
                        }
                    }
                }

                locales_downloaded.push(internal_locale);
            }
        }
    }

    if !skip_screenshots || urls_only {
        eprintln!(
            "Fetching screenshots{}...",
            if urls_only { " (URLs only)" } else { "" }
        );

        // Get all version localizations for screenshot sets
        let version_locs: Value = client
            .get(
                &format!("/appStoreVersions/{version_id}/appStoreVersionLocalizations"),
                &[("limit", "100")],
            )
            .await?;

        if let Some(locs) = version_locs["data"].as_array() {
            for loc in locs {
                let loc_id = loc["id"].as_str().unwrap_or("");
                let asc_locale = loc["attributes"]["locale"].as_str().unwrap_or("en-US");
                let internal_locale = asc_to_internal_locale(asc_locale);

                // Get screenshot sets for this localization
                let sets: Value = client
                    .get(
                        &format!("/appStoreVersionLocalizations/{loc_id}/appScreenshotSets"),
                        &[("limit", "50")],
                    )
                    .await?;

                if let Some(set_arr) = sets["data"].as_array() {
                    for set in set_arr {
                        let set_id = set["id"].as_str().unwrap_or("");
                        let display_type = set["attributes"]["screenshotDisplayType"]
                            .as_str()
                            .unwrap_or("");
                        let dir_name = display_type_to_dir(display_type);

                        // Get screenshots in this set
                        let screenshots: Value = client
                            .get(
                                &format!("/appScreenshotSets/{set_id}/appScreenshots"),
                                &[("limit", "10")],
                            )
                            .await?;

                        if let Some(ss_arr) = screenshots["data"].as_array() {
                            for (idx, ss) in ss_arr.iter().enumerate() {
                                if let Some(url) =
                                    ss["attributes"]["imageAsset"]["templateUrl"].as_str()
                                {
                                    // Replace template placeholders with actual dimensions
                                    let width = ss["attributes"]["imageAsset"]["width"]
                                        .as_u64()
                                        .unwrap_or(0);
                                    let height = ss["attributes"]["imageAsset"]["height"]
                                        .as_u64()
                                        .unwrap_or(0);

                                    let download_url = url
                                        .replace("{w}", &width.to_string())
                                        .replace("{h}", &height.to_string())
                                        .replace("{f}", "png");

                                    if urls_only {
                                        // Collect URLs instead of downloading
                                        screenshot_urls
                                            .entry(internal_locale.clone())
                                            .or_default()
                                            .entry(dir_name.to_string())
                                            .or_default()
                                            .push(download_url);
                                        screenshots_downloaded += 1;
                                    } else if !skip_screenshots {
                                        // Create screenshots directory and download
                                        let ss_dir = output_dir
                                            .join(&internal_locale)
                                            .join("screenshots")
                                            .join(dir_name);
                                        fs::create_dir_all(&ss_dir).await?;

                                        let filename = format!("{:02}.png", idx + 1);
                                        let file_path = ss_dir.join(&filename);

                                        match download_image(&download_url, &file_path).await {
                                            Ok(_) => {
                                                screenshots_downloaded += 1;
                                                eprintln!(
                                                    "  Downloaded: {}/{}/screenshots/{}/{}",
                                                    internal_locale, asc_locale, dir_name, filename
                                                );
                                            }
                                            Err(e) => {
                                                eprintln!("  Failed to download screenshot: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result = json!({
        "success": true,
        "app_id": app_id,
        "locales_downloaded": locales_downloaded,
        "screenshots_downloaded": screenshots_downloaded,
        "output_dir": output_dir.to_string_lossy()
    });

    if urls_only && !screenshot_urls.is_empty() {
        result["screenshot_urls"] = json!(screenshot_urls);
    }

    Ok(result)
}

async fn download_image(url: &str, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status(), url).into());
    }

    let bytes = response.bytes().await?;
    fs::write(path, bytes).await?;
    Ok(())
}

async fn handle_push(
    bundle_id: &str,
    metadata_dir: &PathBuf,
    skip_screenshots: bool,
    skip_metadata: bool,
    client: &AppleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    eprintln!("Looking up app: {}", bundle_id);
    let app_id = lookup_app_by_bundle_id(bundle_id, client).await?;
    eprintln!("Found app ID: {}", app_id);

    // Get or create an editable version (creates new version if current is not editable)
    let version = get_or_create_editable_version(&app_id, client).await?;
    let version_id = version["id"].as_str().ok_or("Version ID not found")?;
    let version_string = version["attributes"]["versionString"]
        .as_str()
        .unwrap_or("unknown");
    eprintln!("Using version: {} ({})", version_id, version_string);

    let (app_info, app_info_editable) = get_app_info(&app_id, client).await?;
    let app_info_id = app_info["id"].as_str().ok_or("App Info ID not found")?;

    if !app_info_editable {
        eprintln!("Note: App info (name/subtitle) may not be editable in current state");
    }

    // Get existing localizations
    let version_locs: Value = client
        .get(
            &format!("/appStoreVersions/{version_id}/appStoreVersionLocalizations"),
            &[("limit", "100")],
        )
        .await?;

    let app_info_locs: Value = client
        .get(
            &format!("/appInfos/{app_info_id}/appInfoLocalizations"),
            &[("limit", "100")],
        )
        .await?;

    // Build maps of existing localizations by locale
    let mut version_loc_map: HashMap<String, String> = HashMap::new();
    if let Some(locs) = version_locs["data"].as_array() {
        for loc in locs {
            if let (Some(locale), Some(id)) =
                (loc["attributes"]["locale"].as_str(), loc["id"].as_str())
            {
                version_loc_map.insert(locale.to_string(), id.to_string());
            }
        }
    }

    let mut app_info_loc_map: HashMap<String, String> = HashMap::new();
    if let Some(locs) = app_info_locs["data"].as_array() {
        for loc in locs {
            if let (Some(locale), Some(id)) =
                (loc["attributes"]["locale"].as_str(), loc["id"].as_str())
            {
                app_info_loc_map.insert(locale.to_string(), id.to_string());
            }
        }
    }

    let mut locales_pushed = Vec::new();
    let mut screenshots_uploaded = 0u32;

    // Scan metadata directory for locale folders
    let mut entries = fs::read_dir(metadata_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let internal_locale = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip non-locale directories
        if internal_locale.is_empty()
            || internal_locale == "screenshots"
            || internal_locale.starts_with('.')
        {
            continue;
        }

        let asc_locale = internal_to_asc_locale(&internal_locale);
        eprintln!(
            "Processing locale: {} (ASC: {})",
            internal_locale, asc_locale
        );

        if !skip_metadata {
            // Read metadata files
            let name = read_file_if_exists(&path.join("name.txt")).await;
            let subtitle = read_file_if_exists(&path.join("subtitle.txt")).await;
            let description = read_file_if_exists(&path.join("description.txt")).await;
            let keywords = read_file_if_exists(&path.join("keywords.txt")).await;
            let whats_new = read_file_if_exists(&path.join("release_notes.txt")).await;
            let promo_text = read_file_if_exists(&path.join("promotional_text.txt")).await;
            let marketing_url = read_file_if_exists(&path.join("marketing_url.txt")).await;
            let support_url = read_file_if_exists(&path.join("support_url.txt")).await;

            // Update or create version localization
            if description.is_some()
                || keywords.is_some()
                || whats_new.is_some()
                || promo_text.is_some()
                || marketing_url.is_some()
                || support_url.is_some()
            {
                let mut attrs = json!({});
                if let Some(v) = &description {
                    attrs["description"] = json!(v);
                }
                if let Some(v) = &keywords {
                    attrs["keywords"] = json!(v);
                }
                if let Some(v) = &whats_new {
                    attrs["whatsNew"] = json!(v);
                }
                if let Some(v) = &promo_text {
                    attrs["promotionalText"] = json!(v);
                }
                if let Some(v) = &marketing_url {
                    attrs["marketingUrl"] = json!(v);
                }
                if let Some(v) = &support_url {
                    attrs["supportUrl"] = json!(v);
                }

                if let Some(loc_id) = version_loc_map.get(&asc_locale) {
                    // Update existing
                    let body = json!({
                        "data": {
                            "type": "appStoreVersionLocalizations",
                            "id": loc_id,
                            "attributes": attrs
                        }
                    });
                    if let Err(e) = client
                        .patch(&format!("/appStoreVersionLocalizations/{loc_id}"), &body)
                        .await
                    {
                        eprintln!(
                            "  Warning: Could not update version localization for {}: {}",
                            asc_locale, e
                        );
                    } else {
                        eprintln!("  Updated version localization");
                    }
                } else {
                    // Create new
                    attrs["locale"] = json!(asc_locale);
                    let body = json!({
                        "data": {
                            "type": "appStoreVersionLocalizations",
                            "attributes": attrs,
                            "relationships": {
                                "appStoreVersion": {
                                    "data": {
                                        "type": "appStoreVersions",
                                        "id": version_id
                                    }
                                }
                            }
                        }
                    });
                    match client.post("/appStoreVersionLocalizations", &body).await {
                        Ok(result) => {
                            if let Some(id) = result["data"]["id"].as_str() {
                                version_loc_map.insert(asc_locale.clone(), id.to_string());
                            }
                            eprintln!("  Created version localization");
                        }
                        Err(e) => {
                            eprintln!(
                                "  Warning: Could not create version localization for {}: {}",
                                asc_locale, e
                            );
                        }
                    }
                }
            }

            // Update or create app info localization (name, subtitle)
            // Skip if app info is not editable
            if (name.is_some() || subtitle.is_some()) && app_info_editable {
                let mut attrs = json!({});
                if let Some(v) = &name {
                    attrs["name"] = json!(v);
                }
                if let Some(v) = &subtitle {
                    attrs["subtitle"] = json!(v);
                }

                if let Some(loc_id) = app_info_loc_map.get(&asc_locale) {
                    // Update existing - use match to handle errors gracefully
                    let body = json!({
                        "data": {
                            "type": "appInfoLocalizations",
                            "id": loc_id,
                            "attributes": attrs
                        }
                    });
                    match client
                        .patch(&format!("/appInfoLocalizations/{loc_id}"), &body)
                        .await
                    {
                        Ok(_) => eprintln!("  Updated app info localization"),
                        Err(e) => eprintln!("  Warning: Could not update name/subtitle: {}", e),
                    }
                } else {
                    // Create new - handle errors gracefully
                    attrs["locale"] = json!(asc_locale);
                    let body = json!({
                        "data": {
                            "type": "appInfoLocalizations",
                            "attributes": attrs,
                            "relationships": {
                                "appInfo": {
                                    "data": {
                                        "type": "appInfos",
                                        "id": app_info_id
                                    }
                                }
                            }
                        }
                    });
                    match client.post("/appInfoLocalizations", &body).await {
                        Ok(result) => {
                            if let Some(id) = result["data"]["id"].as_str() {
                                app_info_loc_map.insert(asc_locale.clone(), id.to_string());
                            }
                            eprintln!("  Created app info localization");
                        }
                        Err(e) => eprintln!("  Warning: Could not create name/subtitle: {}", e),
                    }
                }
            } else if name.is_some() || subtitle.is_some() {
                eprintln!("  Skipping name/subtitle (app info not editable)");
            }

            locales_pushed.push(internal_locale.clone());
        }

        if !skip_screenshots {
            // Handle screenshots
            let screenshots_dir = path.join("screenshots");
            if screenshots_dir.exists() {
                // Get or create version localization for screenshots
                let loc_id = if let Some(id) = version_loc_map.get(&asc_locale) {
                    Some(id.clone())
                } else {
                    // Create localization first
                    let body = json!({
                        "data": {
                            "type": "appStoreVersionLocalizations",
                            "attributes": {
                                "locale": asc_locale
                            },
                            "relationships": {
                                "appStoreVersion": {
                                    "data": {
                                        "type": "appStoreVersions",
                                        "id": version_id
                                    }
                                }
                            }
                        }
                    });
                    match client.post("/appStoreVersionLocalizations", &body).await {
                        Ok(result) => {
                            if let Some(id) = result["data"]["id"].as_str() {
                                let id_str = id.to_string();
                                version_loc_map.insert(asc_locale.clone(), id_str.clone());
                                Some(id_str)
                            } else {
                                None
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "  Warning: Could not create localization for {}: {}",
                                asc_locale, e
                            );
                            None
                        }
                    }
                };

                if let Some(loc_id) = loc_id {
                    // Get existing screenshot sets
                    let sets: Value = client
                        .get(
                            &format!("/appStoreVersionLocalizations/{loc_id}/appScreenshotSets"),
                            &[("limit", "50")],
                        )
                        .await?;

                    let mut set_map: HashMap<String, String> = HashMap::new();
                    if let Some(set_arr) = sets["data"].as_array() {
                        for set in set_arr {
                            if let (Some(dt), Some(id)) = (
                                set["attributes"]["screenshotDisplayType"].as_str(),
                                set["id"].as_str(),
                            ) {
                                set_map.insert(dt.to_string(), id.to_string());
                            }
                        }
                    }

                    // Process each device type directory
                    let mut ss_entries = fs::read_dir(&screenshots_dir).await?;
                    while let Some(ss_entry) = ss_entries.next_entry().await? {
                        let ss_path = ss_entry.path();
                        if !ss_path.is_dir() {
                            continue;
                        }

                        let dir_name = ss_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let display_type = dir_to_display_type(dir_name);

                        // Get or create screenshot set
                        let set_id = if let Some(id) = set_map.get(display_type) {
                            Some(id.clone())
                        } else {
                            let body = json!({
                                "data": {
                                    "type": "appScreenshotSets",
                                    "attributes": {
                                        "screenshotDisplayType": display_type
                                    },
                                    "relationships": {
                                        "appStoreVersionLocalization": {
                                            "data": {
                                                "type": "appStoreVersionLocalizations",
                                                "id": loc_id
                                            }
                                        }
                                    }
                                }
                            });
                            match client.post("/appScreenshotSets", &body).await {
                                Ok(result) => {
                                    if let Some(id) = result["data"]["id"].as_str() {
                                        let id_str = id.to_string();
                                        set_map.insert(display_type.to_string(), id_str.clone());
                                        Some(id_str)
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "  Warning: Could not create screenshot set for {}: {}",
                                        display_type, e
                                    );
                                    None
                                }
                            }
                        };

                        if let Some(set_id) = set_id {
                            // Delete existing screenshots in the set
                            let existing: Value = client
                                .get(
                                    &format!("/appScreenshotSets/{set_id}/appScreenshots"),
                                    &[("limit", "10")],
                                )
                                .await?;

                            if let Some(existing_arr) = existing["data"].as_array() {
                                for ss in existing_arr {
                                    if let Some(ss_id) = ss["id"].as_str() {
                                        let _ = client
                                            .delete(&format!("/appScreenshots/{ss_id}"))
                                            .await;
                                    }
                                }
                            }

                            // Upload new screenshots
                            let mut images: Vec<PathBuf> = Vec::new();
                            let mut img_entries = fs::read_dir(&ss_path).await?;
                            while let Some(img_entry) = img_entries.next_entry().await? {
                                let img_path = img_entry.path();
                                if img_path
                                    .extension()
                                    .map(|e| e == "png" || e == "jpg" || e == "jpeg")
                                    .unwrap_or(false)
                                {
                                    images.push(img_path);
                                }
                            }
                            images.sort();

                            let mut uploaded_ids = Vec::new();
                            for (idx, img_path) in images.iter().take(10).enumerate() {
                                let filename = img_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("screenshot.png");

                                match upload_screenshot(client, &set_id, img_path, filename).await {
                                    Ok(screenshot_id) => {
                                        uploaded_ids.push(screenshot_id);
                                        screenshots_uploaded += 1;
                                        eprintln!(
                                            "  Uploaded: {}/{} ({}/{})",
                                            dir_name,
                                            filename,
                                            idx + 1,
                                            images.len().min(10)
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("  Failed to upload {}: {}", filename, e);
                                    }
                                }
                            }

                            // Reorder screenshots if we uploaded multiple
                            if uploaded_ids.len() > 1 {
                                let data: Vec<Value> = uploaded_ids
                                    .iter()
                                    .map(|id| {
                                        json!({
                                            "type": "appScreenshots",
                                            "id": id
                                        })
                                    })
                                    .collect();
                                let body = json!({ "data": data });
                                let _ = client
                                    .patch(
                                        &format!("/appScreenshotSets/{set_id}/relationships/appScreenshots"),
                                        &body,
                                    )
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "success": true,
        "app_id": app_id,
        "locales_pushed": locales_pushed,
        "screenshots_uploaded": screenshots_uploaded
    }))
}

async fn read_file_if_exists(path: &PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .await
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn upload_screenshot(
    client: &AppleClient,
    set_id: &str,
    file_path: &PathBuf,
    filename: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let file_size = fs::metadata(file_path).await?.len();

    // Step 1: Reserve the screenshot
    let reservation = json!({
        "data": {
            "type": "appScreenshots",
            "attributes": {
                "fileName": filename,
                "fileSize": file_size
            },
            "relationships": {
                "appScreenshotSet": {
                    "data": {
                        "type": "appScreenshotSets",
                        "id": set_id
                    }
                }
            }
        }
    });

    let reserved: Value = client.post("/appScreenshots", &reservation).await?;
    let screenshot_id = reserved["data"]["id"]
        .as_str()
        .ok_or("no screenshot id in reservation response")?;

    // Step 2: Upload the asset
    let upload_ops = &reserved["data"]["attributes"]["uploadOperations"];
    let file_bytes = fs::read(file_path).await?;

    if let Some(ops) = upload_ops.as_array() {
        let http_client = reqwest::Client::new();
        for op in ops {
            let url = op["url"].as_str().ok_or("missing upload url")?;
            let offset = op["offset"].as_u64().unwrap_or(0) as usize;
            let length = op["length"].as_u64().unwrap_or(file_bytes.len() as u64) as usize;
            let chunk = &file_bytes[offset..std::cmp::min(offset + length, file_bytes.len())];

            let mut req = http_client.put(url);
            if let Some(headers) = op["requestHeaders"].as_array() {
                for h in headers {
                    if let (Some(name), Some(value)) = (h["name"].as_str(), h["value"].as_str()) {
                        req = req.header(name, value);
                    }
                }
            }
            req.body(chunk.to_vec()).send().await?;
        }
    }

    // Step 3: Commit the upload
    let commit_body = json!({
        "data": {
            "type": "appScreenshots",
            "id": screenshot_id,
            "attributes": {
                "uploaded": true,
                "sourceFileChecksum": reserved["data"]["attributes"]["sourceFileChecksum"]
            }
        }
    });

    client
        .patch(&format!("/appScreenshots/{screenshot_id}"), &commit_body)
        .await?;

    Ok(screenshot_id.to_string())
}
