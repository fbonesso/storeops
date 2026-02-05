//! Google Play Store sync commands for batch metadata and screenshot operations.
//!
//! Provides pull (download) and push (upload) functionality for app metadata and screenshots
//! across all locales in a single operation.

use clap::Subcommand;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;

use crate::api::google_client::GoogleClient;

/// Google Play locale codes mapped to internal standardized codes.
/// Format: "gp_locale" -> "internal_locale"
fn gp_to_internal_locale(gp_locale: &str) -> String {
    match gp_locale {
        "pl-PL" => "pl".to_string(),
        "sv-SE" => "sv".to_string(),
        "es-419" => "es-MX".to_string(), // Latin America Spanish -> our Mexican Spanish
        _ => gp_locale.to_string(),
    }
}

/// Internal locale codes mapped to Google Play codes.
/// Format: "internal_locale" -> "gp_locale"
fn internal_to_gp_locale(internal_locale: &str) -> String {
    match internal_locale {
        "pl" => "pl-PL".to_string(),
        "sv" => "sv-SE".to_string(),
        "es-MX" => "es-419".to_string(), // Mexican Spanish -> Latin America Spanish
        _ => internal_locale.to_string(),
    }
}

/// Google Play image types for screenshots
const SCREENSHOT_TYPES: &[(&str, &str)] = &[
    ("phoneScreenshots", "phoneScreenshots"),
    ("sevenInchScreenshots", "sevenInchScreenshots"),
    ("tenInchScreenshots", "tenInchScreenshots"),
    ("tvScreenshots", "tvScreenshots"),
    ("wearScreenshots", "wearScreenshots"),
];

/// Google Play image types for graphics
const GRAPHIC_TYPES: &[(&str, &str)] = &[
    ("featureGraphic", "featureGraphic"),
    ("icon", "icon"),
    ("tvBanner", "tvBanner"),
];

#[derive(Subcommand)]
pub enum SyncCommand {
    /// Pull (download) all metadata and screenshots for an app
    Pull {
        /// Android Package Name (e.g., com.example.app)
        package_name: String,
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
        /// Android Package Name (e.g., com.example.app)
        package_name: String,
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
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        SyncCommand::Pull {
            package_name,
            output_dir,
            skip_screenshots,
            skip_metadata,
            urls_only,
        } => {
            handle_pull(
                package_name,
                output_dir,
                *skip_screenshots,
                *skip_metadata,
                *urls_only,
                client,
            )
            .await
        }
        SyncCommand::Push {
            package_name,
            metadata_dir,
            skip_screenshots,
            skip_metadata,
        } => {
            handle_push(
                package_name,
                metadata_dir,
                *skip_screenshots,
                *skip_metadata,
                client,
            )
            .await
        }
    }
}

async fn handle_pull(
    package_name: &str,
    output_dir: &PathBuf,
    skip_screenshots: bool,
    skip_metadata: bool,
    urls_only: bool,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    eprintln!("Creating edit for package: {}", package_name);

    // Create an edit session
    let edit: Value = client
        .post(&format!("/{package_name}/edits"), &json!({}))
        .await?;
    let edit_id = edit["id"].as_str().ok_or("no edit id")?;
    eprintln!("Edit ID: {}", edit_id);

    // Create output directory
    fs::create_dir_all(output_dir).await?;

    let mut locales_downloaded = Vec::new();
    let mut screenshots_downloaded = 0u32;
    // For urls_only mode: locale -> device -> [urls]
    let mut screenshot_urls: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    if !skip_metadata {
        // Get all store listings
        eprintln!("Fetching store listings...");
        let listings: Value = client
            .get(&format!("/{package_name}/edits/{edit_id}/listings"), &[])
            .await?;

        if let Some(listings_arr) = listings["listings"].as_array() {
            for listing in listings_arr {
                let gp_locale = listing["language"].as_str().unwrap_or("en-US");
                let internal_locale = gp_to_internal_locale(gp_locale);
                let locale_dir = output_dir.join(&internal_locale);
                fs::create_dir_all(&locale_dir).await?;

                // title.txt
                if let Some(title) = listing["title"].as_str() {
                    if !title.is_empty() {
                        fs::write(locale_dir.join("title.txt"), title).await?;
                    }
                }

                // short_description.txt
                if let Some(short_desc) = listing["shortDescription"].as_str() {
                    if !short_desc.is_empty() {
                        fs::write(locale_dir.join("short_description.txt"), short_desc).await?;
                    }
                }

                // full_description.txt
                if let Some(full_desc) = listing["fullDescription"].as_str() {
                    if !full_desc.is_empty() {
                        fs::write(locale_dir.join("full_description.txt"), full_desc).await?;
                    }
                }

                // video.txt (YouTube URL)
                if let Some(video) = listing["video"].as_str() {
                    if !video.is_empty() {
                        fs::write(locale_dir.join("video.txt"), video).await?;
                    }
                }

                locales_downloaded.push(internal_locale.clone());
                eprintln!("  Downloaded metadata for: {}", internal_locale);
            }
        }
    }

    if !skip_screenshots || urls_only {
        eprintln!(
            "Fetching images{}...",
            if urls_only { " (URLs only)" } else { "" }
        );

        // Get all store listings to know which locales have content
        let listings: Value = client
            .get(&format!("/{package_name}/edits/{edit_id}/listings"), &[])
            .await?;

        let locales: Vec<String> = if let Some(listings_arr) = listings["listings"].as_array() {
            listings_arr
                .iter()
                .filter_map(|l| l["language"].as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        };

        for gp_locale in &locales {
            let internal_locale = gp_to_internal_locale(gp_locale);

            // Download screenshots
            for (image_type, dir_name) in SCREENSHOT_TYPES {
                let response: Value = match client
                    .get(
                        &format!(
                            "/{package_name}/edits/{edit_id}/listings/{gp_locale}/{image_type}"
                        ),
                        &[],
                    )
                    .await
                {
                    Ok(v) => v,
                    Err(_) => continue, // No images of this type
                };

                // API returns {"images": [...]} not a direct array
                if let Some(images_arr) = response["images"].as_array() {
                    if images_arr.is_empty() {
                        continue;
                    }

                    for (idx, img) in images_arr.iter().enumerate() {
                        if let Some(url) = img["url"].as_str() {
                            if urls_only {
                                // Collect URLs instead of downloading
                                screenshot_urls
                                    .entry(internal_locale.clone())
                                    .or_default()
                                    .entry(dir_name.to_string())
                                    .or_default()
                                    .push(url.to_string());
                                screenshots_downloaded += 1;
                            } else if !skip_screenshots {
                                // Download the image
                                let images_dir = output_dir.join(&internal_locale).join("images");
                                let ss_dir = images_dir.join(dir_name);
                                fs::create_dir_all(&ss_dir).await?;

                                let filename = format!("{:02}.png", idx + 1);
                                let file_path = ss_dir.join(&filename);

                                match download_image(url, &file_path).await {
                                    Ok(_) => {
                                        screenshots_downloaded += 1;
                                        eprintln!(
                                            "  Downloaded: {}/{}/{}",
                                            internal_locale, dir_name, filename
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("  Failed to download: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Download/collect graphics (feature graphic, icon, etc.)
            for (image_type, file_name) in GRAPHIC_TYPES {
                let response: Value = match client
                    .get(
                        &format!(
                            "/{package_name}/edits/{edit_id}/listings/{gp_locale}/{image_type}"
                        ),
                        &[],
                    )
                    .await
                {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                // API returns {"images": [...]} not a direct array
                if let Some(images_arr) = response["images"].as_array() {
                    if let Some(img) = images_arr.first() {
                        if let Some(url) = img["url"].as_str() {
                            if urls_only {
                                // Collect URL for graphics too
                                screenshot_urls
                                    .entry(internal_locale.clone())
                                    .or_default()
                                    .entry(file_name.to_string())
                                    .or_default()
                                    .push(url.to_string());
                                screenshots_downloaded += 1;
                            } else if !skip_screenshots {
                                let images_dir = output_dir.join(&internal_locale).join("images");
                                fs::create_dir_all(&images_dir).await?;
                                let file_path = images_dir.join(format!("{}.png", file_name));

                                match download_image(url, &file_path).await {
                                    Ok(_) => {
                                        screenshots_downloaded += 1;
                                        eprintln!(
                                            "  Downloaded: {}/{}.png",
                                            internal_locale, file_name
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("  Failed to download {}: {}", file_name, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Discard the edit (we were only reading)
    let _ = client
        .delete_path(&format!("/{package_name}/edits/{edit_id}"))
        .await;

    let mut result = json!({
        "success": true,
        "package_name": package_name,
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
    package_name: &str,
    metadata_dir: &PathBuf,
    skip_screenshots: bool,
    skip_metadata: bool,
    client: &GoogleClient,
) -> Result<Value, Box<dyn std::error::Error>> {
    eprintln!(
        "PUSH START: Package={} Dir={:?}",
        package_name, metadata_dir
    );

    // Create an edit session
    let edit: Value = client
        .post(&format!("/{package_name}/edits"), &json!({}))
        .await?;
    let edit_id = edit["id"].as_str().ok_or("no edit id")?;
    eprintln!("Created Edit Session: {}", edit_id);

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
            || internal_locale == "images"
            || internal_locale.starts_with('.')
        {
            continue;
        }

        let gp_locale = internal_to_gp_locale(&internal_locale);
        eprintln!("Processing locale: {} (GP: {})", internal_locale, gp_locale);

        let mut locale_updated = false;

        if !skip_metadata {
            // Read metadata files
            let title = read_file_if_exists(&path.join("title.txt")).await;
            let short_description = read_file_if_exists(&path.join("short_description.txt")).await;
            let full_description = read_file_if_exists(&path.join("full_description.txt")).await;
            let video = read_file_if_exists(&path.join("video.txt")).await;

            // Update store listing if we have any content
            if title.is_some()
                || short_description.is_some()
                || full_description.is_some()
                || video.is_some()
            {
                let mut body = json!({ "language": gp_locale });

                if let Some(v) = &title {
                    body["title"] = json!(v);
                }
                if let Some(v) = &short_description {
                    body["shortDescription"] = json!(v);
                }
                if let Some(v) = &full_description {
                    body["fullDescription"] = json!(v);
                }
                if let Some(v) = &video {
                    body["video"] = json!(v);
                }

                match client
                    .put(
                        &format!("/{package_name}/edits/{edit_id}/listings/{gp_locale}"),
                        &body,
                    )
                    .await
                {
                    Ok(_) => {
                        eprintln!("  Updated store listing");
                        locale_updated = true;
                    }
                    Err(e) => {
                        eprintln!(
                            "  Warning: Could not update listing for locale {}: {}",
                            gp_locale, e
                        );
                    }
                }
            }
        }

        if !skip_screenshots {
            let images_dir = path.join("images");
            if images_dir.exists() {
                // Upload screenshots
                for (image_type, dir_name) in SCREENSHOT_TYPES {
                    let ss_dir = images_dir.join(dir_name);
                    if !ss_dir.exists() {
                        continue;
                    }

                    // Delete existing screenshots of this type
                    match client
                        .delete_path(&format!(
                            "/{package_name}/edits/{edit_id}/listings/{gp_locale}/{image_type}"
                        ))
                        .await
                    {
                        Ok(_) => eprintln!("  Deleted existing {}", image_type),
                        Err(e) => {
                            eprintln!("  Warning: Could not delete existing {}: {}", image_type, e)
                        }
                    }

                    // Get sorted list of images
                    let mut images: Vec<PathBuf> = Vec::new();
                    let mut img_entries = fs::read_dir(&ss_dir).await?;
                    while let Some(img_entry) = img_entries.next_entry().await? {
                        let img_path = img_entry.path();
                        if img_path
                            .extension()
                            .map(|e| {
                                let e_str = e.to_string_lossy().to_lowercase();
                                e_str == "png" || e_str == "jpg" || e_str == "jpeg"
                            })
                            .unwrap_or(false)
                        {
                            images.push(img_path);
                        }
                    }
                    images.sort();

                    eprintln!(
                        "  Found {} images to upload for {}",
                        images.len(),
                        image_type
                    );

                    // Upload images (max 8 per type)
                    for (idx, img_path) in images.iter().take(8).enumerate() {
                        let filename = img_path.file_name().unwrap_or_default().to_string_lossy();
                        match client
                            .upload_image(
                                package_name,
                                edit_id,
                                &gp_locale,
                                image_type,
                                img_path.to_string_lossy().as_ref(),
                            )
                            .await
                        {
                            Ok(_) => {
                                screenshots_uploaded += 1;
                                locale_updated = true;
                                eprintln!(
                                    "  Uploaded: {}/{} ({}/{})",
                                    dir_name,
                                    filename,
                                    idx + 1,
                                    images.len().min(8)
                                );
                            }
                            Err(e) => {
                                eprintln!(
                                    "  Failed to upload {} (type: {}): {}",
                                    filename, image_type, e
                                );
                            }
                        }
                    }
                }

                // Upload graphics (feature graphic, icon, etc.)
                for (image_type, file_name) in GRAPHIC_TYPES {
                    // Check for various extensions
                    let possible_paths = [
                        images_dir.join(format!("{}.png", file_name)),
                        images_dir.join(format!("{}.jpg", file_name)),
                        images_dir.join(format!("{}.jpeg", file_name)),
                    ];

                    for img_path in &possible_paths {
                        if img_path.exists() {
                            // Delete existing
                            let _ = client
                                .delete_path(&format!(
                                    "/{package_name}/edits/{edit_id}/listings/{gp_locale}/{image_type}"
                                ))
                                .await;

                            eprintln!("  Uploading graphic: {}", image_type);
                            match client
                                .upload_image(
                                    package_name,
                                    edit_id,
                                    &gp_locale,
                                    image_type,
                                    img_path.to_string_lossy().as_ref(),
                                )
                                .await
                            {
                                Ok(_) => {
                                    screenshots_uploaded += 1;
                                    locale_updated = true;
                                    eprintln!("  Uploaded: {}", file_name);
                                }
                                Err(e) => {
                                    eprintln!("  Failed to upload {}: {}", file_name, e);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        if locale_updated {
            locales_pushed.push(internal_locale.clone());
        }
        eprintln!("Locale Session SUCCESS: {}", internal_locale);
    }

    if locales_pushed.is_empty() {
        return Err("No locales found to push in metadata_dir".into());
    }

    // Commit the edit
    // Use changesNotSentForReview=true to allow commits when managed publishing is enabled
    // or when the app is in a state that doesn't allow automatic review submission
    eprintln!("Committing changes...");
    client
        .post(
            &format!("/{package_name}/edits/{edit_id}:commit?changesNotSentForReview=true"),
            &json!({}),
        )
        .await?;

    eprintln!("COMMIT SUCCESSFUL.");
    Ok(json!({
        "success": true,
        "package_name": package_name,
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
