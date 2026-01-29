# AGENTS.md

## What is StoreOps

StoreOps (`storeops`) is a CLI for managing the full App Store Connect and Google Play Store lifecycle. It outputs JSON by default, accepts all input via flags (no interactive prompts), and returns structured errors on stderr.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/fbonesso/storeops/main/install.sh | sh
```

Or update an existing install:

```bash
storeops update
```

## Authentication

Before running any store command, set up credentials:

```bash
# Apple (App Store Connect API key)
storeops auth login --store apple \
  --key-id XXXXXXXXXX \
  --issuer-id XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX \
  --key-path /path/to/AuthKey.p8 \
  --name my-apple

# Google (service account)
storeops auth login --store google \
  --service-account /path/to/service-account.json \
  --name my-google

# Check status
storeops auth status

# Switch profile
storeops auth switch my-apple
```

Credentials can also be set via environment variables:
- Apple: `STOREOPS_APPLE_KEY_ID`, `STOREOPS_APPLE_ISSUER_ID`, `STOREOPS_APPLE_KEY_PATH`
- Google: `STOREOPS_GOOGLE_SERVICE_ACCOUNT`

Use `--profile <name>` on any command to override the active profile.

## Command Pattern

All commands follow: `storeops <store> <resource> <action> [flags]`

Actions are consistent across resources: `list`, `info`/`get`, `create`, `update`, `delete`.

## Global Flags

| Flag | Description |
|------|-------------|
| `--output json\|table\|markdown` | Output format (default: json) |
| `--pretty` | Pretty-print JSON |
| `--profile <name>` | Use a specific auth profile |
| `--limit <N>` | Pagination limit |
| `--next <cursor>` | Pagination cursor |
| `--paginate` | Auto-fetch all pages |
| `--timeout <secs>` | Request timeout (default: 30) |
| `--verbose` | Debug logging |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User error |
| 2 | Auth error |
| 3 | API error |
| 4 | Network error |

## Chaining Commands

Always parse JSON output to extract IDs for downstream commands:

```bash
APP_ID=$(storeops apple apps list | jq -r '.data[0].id')
storeops apple versions list --app-id "$APP_ID"
```

## Core Workflows

### Publish an iOS Version

```bash
storeops apple apps list
storeops apple builds list --app-id APP_ID --limit 5
storeops apple versions create --app-id APP_ID --version "2.1.0" --platform ios
storeops apple metadata localizations update --version-id VER_ID --locale en-US \
  --whats-new "Bug fixes." --description "The best app."
storeops apple versions update --version-id VER_ID --build-id BUILD_ID
storeops apple submit APP_ID --version "2.1.0"
```

### Publish to Google Play

```bash
storeops google builds upload --app-id com.example.app --file app-release.aab
storeops google tracks update --app-id com.example.app --track production \
  --version-code 42 --rollout-fraction 0.1
storeops google listings update --app-id com.example.app --locale en-US \
  --title "My App" --short-description "Short." --full-description "Full."
storeops google submit --app-id com.example.app
```

### Upload Screenshots (Apple)

```bash
storeops apple screenshots sets list --version-id VER_ID --locale en-US
storeops apple screenshots sets create --version-id VER_ID --locale en-US \
  --display-type APP_IPHONE_67
storeops apple screenshots images upload --set-id SET_ID --file screenshot.png
```

### Upload Images (Google)

```bash
storeops google images upload --app-id com.example.app --locale en-US \
  --image-type phoneScreenshots --file screenshot.png
```

### Manage TestFlight

```bash
storeops apple testflight groups list --app-id APP_ID
storeops apple testflight testers add --group-id GROUP_ID \
  --email tester@example.com --first-name Jane --last-name Doe
```

### In-App Purchases

```bash
storeops apple iap list --app-id APP_ID
storeops apple iap create --app-id APP_ID --product-id com.example.coins100 \
  --type consumable --reference-name "100 Coins"
storeops apple iap localizations create --iap-id IAP_ID --locale en-US \
  --display-name "100 Coins" --description "Buy 100 coins"
storeops apple iap submit --iap-id IAP_ID
```

### Reviews

```bash
storeops apple reviews list --app-id APP_ID --limit 20
storeops apple reviews respond --review-id REV_ID --response "Thanks!"

storeops google reviews list --app-id com.example.app
storeops google reviews reply --review-id REV_ID --reply "Thank you!"
```

### Pricing and Availability

```bash
storeops apple pricing get --app-id APP_ID
storeops apple pricing set --app-id APP_ID --price-point POINT_ID
storeops apple availability set --app-id APP_ID --territories US,GB,DE,JP
```

### Phased Release

```bash
storeops apple phased-release create --version-id VER_ID
storeops apple phased-release update --version-id VER_ID --state PAUSE
storeops apple phased-release delete --version-id VER_ID
```

## Error Handling

When a command fails:
1. Check the exit code (2 = auth, 3 = API, 4 = network).
2. Parse the JSON error from stderr: `storeops ... 2>&1 | jq '.error'`
3. For auth errors, run `storeops auth status` and re-login if needed.
4. For rate limits (HTTP 429), wait and retry.

## Tips for Agents

- Always use `--output json` (the default) and parse with `jq`.
- Use `--paginate` when you need complete lists.
- Chain commands by extracting IDs with `jq -r '.data[].id'`.
- Loop over locales for multi-language updates rather than making separate requests to the user.
- Verify state after mutations: after `submit`, check versions list to confirm status.
- Disable background update checks in CI with `STOREOPS_NO_UPDATE_CHECK=1`.
