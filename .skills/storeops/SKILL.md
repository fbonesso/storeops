---
name: storeops
description: >
  Manage the full Apple App Store Connect and Google Play Store lifecycle using
  the storeops CLI. Use this skill when the user wants to publish
  apps, upload builds, manage TestFlight, update store listings and metadata,
  upload screenshots or previews, handle in-app purchases and subscriptions,
  set pricing, manage phased releases, respond to reviews, or pull analytics.
  The CLI outputs JSON by default and never prompts interactively, making it
  ideal for agent-driven automation.
---

# StoreOps CLI Skill

## When to Use

Activate this skill when the user wants to:
- Publish or submit an app version to Apple or Google
- Upload builds, screenshots, or preview videos
- Update store listings, metadata, or localizations
- Manage TestFlight beta testing or Google Play test tracks
- Handle in-app purchases or subscriptions
- Set pricing, age ratings, or availability
- Respond to app reviews
- Pull sales or analytics data
- Check build status or review status
- Register test devices

## Key Concepts

- **Binary**: `storeops`
- **Output**: JSON by default. Use `--output table` for human-readable, `--pretty` for formatted JSON, `--output markdown` for markdown tables.
- **Profiles**: Named auth profiles let you switch between apps/accounts. Use `--profile <name>` on any command.
- **Pagination**: List commands support `--limit N`, `--next <cursor>`, and `--paginate` (fetch all pages).
- **Exit codes**: 0 = success, 1 = user error, 2 = auth error, 3 = API error, 4 = network error.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/fbonesso/storeops/main/install.sh | sh
```

## Auth Setup

Before any store commands, authenticate:

```bash
# Apple: API key auth (recommended for automation)
storeops auth login --store apple \
  --key-id XXXXXXXXXX \
  --issuer-id XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX \
  --key-path /path/to/AuthKey_XXXXXXXXXX.p8 \
  --name my-apple

# Google: service account auth
storeops auth login --store google \
  --service-account /path/to/service-account.json \
  --name my-google

# Check current auth
storeops auth status

# Switch between saved profiles
storeops auth switch my-apple
```

If auth fails with exit code 2, verify credentials and re-run `storeops auth login`.

## Command Patterns

All commands follow: `storeops <store> <resource> <action> [flags]`

Actions are consistent across resources:
- `list` — list resources (supports `--limit`, `--next`, `--paginate`)
- `info` / `get` — get a single resource by ID
- `create` — create a new resource
- `update` — modify an existing resource
- `delete` — remove a resource

Always capture JSON output for chaining:

```bash
# Apple: get an app ID, then use it downstream
APP_ID=$(storeops apple apps list | jq -r '.data[0].id')
storeops apple versions list --app-id "$APP_ID"

# Google: use known package name (no list-apps endpoint)
storeops google apps info com.example.app
```

## Core Workflows

### 1. Publish a New iOS Version

```bash
# 1. List apps to get the app ID
storeops apple apps list

# 2. Check latest build is processed
storeops apple builds list --app-id APP_ID --limit 5

# 3. Create a new version
storeops apple versions create --app-id APP_ID --version "2.1.0" --platform ios

# 4. Update localizations (repeat per locale)
storeops apple metadata localizations update --version-id VER_ID --locale en-US \
  --whats-new "Bug fixes and performance improvements." \
  --description "The best app ever." \
  --keywords "app,store,keywords"

# 5. Set the build for this version
storeops apple versions update --version-id VER_ID --build-id BUILD_ID

# 6. Submit for review
storeops apple submit APP_ID --version "2.1.0"
```

### 2. Publish to Google Play

```bash
# 1. Upload a new build (AAB)
storeops google builds upload --app-id com.example.app --file app-release.aab

# 2. Assign the build to a track (e.g., production with staged rollout)
storeops google tracks update --app-id com.example.app --track production \
  --version-code 42 --rollout-fraction 0.1

# 3. Update the listing
storeops google listings update --app-id com.example.app --locale en-US \
  --title "My App" --short-description "Short." --full-description "Full."

# 4. Submit (finalize the edit)
storeops google submit --app-id com.example.app
```

### 3. Update Store Listings for All Locales

```bash
# Get current localizations
storeops apple metadata localizations list --version-id VER_ID

# Update each locale
for LOCALE in en-US ja de-DE fr-FR; do
  storeops apple metadata localizations update --version-id VER_ID \
    --locale "$LOCALE" --whats-new "$(cat "notes_${LOCALE}.txt")"
done
```

### 4. Upload Screenshots

```bash
# 1. List existing screenshot sets
storeops apple screenshots sets list --version-id VER_ID --locale en-US

# 2. Create a set for a display type if it doesn't exist
storeops apple screenshots sets create --version-id VER_ID --locale en-US \
  --display-type APP_IPHONE_67

# 3. Upload images to the set
storeops apple screenshots images upload --set-id SET_ID --file screenshot1.png
storeops apple screenshots images upload --set-id SET_ID --file screenshot2.png

# 4. Reorder if needed
storeops apple screenshots images reorder --set-id SET_ID \
  --image-ids '["IMG_1","IMG_2","IMG_3"]'
```

For Google:
```bash
storeops google images upload --app-id com.example.app --locale en-US \
  --image-type phoneScreenshots --file screenshot.png
```

### 5. Manage TestFlight

```bash
# List beta groups
storeops apple testflight groups list --app-id APP_ID

# Add a tester by email
storeops apple testflight testers add --group-id GROUP_ID \
  --email tester@example.com --first-name Jane --last-name Doe

# List testers in a group
storeops apple testflight testers list --group-id GROUP_ID
```

### 6. In-App Purchases and Subscriptions

```bash
# List IAPs
storeops apple iap list --app-id APP_ID

# Create a consumable IAP
storeops apple iap create --app-id APP_ID --product-id com.example.coins100 \
  --type consumable --reference-name "100 Coins"

# Add localization
storeops apple iap localizations create --iap-id IAP_ID --locale en-US \
  --display-name "100 Coins" --description "Buy 100 coins"

# Set price
storeops apple iap prices set --iap-id IAP_ID --price-point PRICE_POINT_ID

# Submit for review
storeops apple iap submit --iap-id IAP_ID

# Subscriptions follow a similar pattern
storeops apple subscriptions groups list --app-id APP_ID
storeops apple subscriptions items create --group-id GRP_ID \
  --product-id com.example.pro_monthly --reference-name "Pro Monthly"
```

### 7. Respond to Reviews

```bash
# List recent reviews
storeops apple reviews list --app-id APP_ID --limit 20

# Respond to a review
storeops apple reviews respond --review-id REV_ID \
  --response "Thanks for the feedback!"

# Google
storeops google reviews list --app-id com.example.app
storeops google reviews reply --review-id REV_ID --reply "Thank you!"
```

### 8. Pricing and Availability

```bash
# Get current price
storeops apple pricing get --app-id APP_ID

# List available price points
storeops apple pricing points --app-id APP_ID

# Set price
storeops apple pricing set --app-id APP_ID --price-point POINT_ID

# Manage availability
storeops apple availability get --app-id APP_ID
storeops apple availability territories --app-id APP_ID
storeops apple availability set --app-id APP_ID --territories US,GB,DE,JP
```

### 9. Phased Release

```bash
storeops apple phased-release create --version-id VER_ID
storeops apple phased-release get --version-id VER_ID
storeops apple phased-release update --version-id VER_ID --state PAUSE
storeops apple phased-release delete --version-id VER_ID
```

## Error Handling

When a command fails:
1. Check the exit code (2 = auth, 3 = API error, 4 = network).
2. Parse the JSON error: `storeops ... 2>&1 | jq '.errors'`
3. For auth errors, run `storeops auth status` then re-login if expired.
4. For rate limits (HTTP 429), wait and retry.
5. For validation errors, the `.errors[].detail` field describes what to fix.

## Tips for Agents

- Always use `--output json` (the default) and parse with `jq` for reliable extraction.
- Use `--paginate` when you need complete lists (e.g., all localizations, all screenshots).
- Chain commands by extracting IDs from JSON output with `jq -r '.data[].id'`.
- When updating multiple locales, loop over them rather than making separate requests to the user.
- Verify state after mutations: after `submit`, check `storeops apple versions list` to confirm status.
- See `references/COMMANDS.md` for the full flag reference and `references/WORKFLOWS.md` for detailed step-by-step guides.
