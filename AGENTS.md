# AGENTS.md

## What is StoreOps

StoreOps (`storeops` / `st`) is a CLI for managing the full App Store Connect and Google Play Store lifecycle. It outputs JSON by default, accepts all input via flags (no interactive prompts), and returns structured errors on stderr.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/fbonesso/storeops/main/install.sh | sh
```

Or update an existing install:

```bash
st update
```

## Authentication

Before running any store command, set up credentials:

```bash
# Apple (App Store Connect API key)
st auth login --store apple \
  --key-id XXXXXXXXXX \
  --issuer-id XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX \
  --key-path /path/to/AuthKey.p8 \
  --name my-apple

# Google (service account)
st auth login --store google \
  --service-account /path/to/service-account.json \
  --name my-google

# Check status
st auth status

# Switch profile
st auth switch my-apple
```

Credentials can also be set via environment variables:
- Apple: `STOREOPS_APPLE_KEY_ID`, `STOREOPS_APPLE_ISSUER_ID`, `STOREOPS_APPLE_KEY_PATH`
- Google: `STOREOPS_GOOGLE_SERVICE_ACCOUNT`

Use `--profile <name>` on any command to override the active profile.

## Command Pattern

All commands follow: `st <store> <resource> <action> [flags]`

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
APP_ID=$(st apple apps list | jq -r '.data[0].id')
st apple versions list --app-id "$APP_ID"
```

## Core Workflows

### Publish an iOS Version

```bash
st apple apps list
st apple builds list --app-id APP_ID --limit 5
st apple versions create --app-id APP_ID --version "2.1.0" --platform ios
st apple metadata localizations update --version-id VER_ID --locale en-US \
  --whats-new "Bug fixes." --description "The best app."
st apple versions update --version-id VER_ID --build-id BUILD_ID
st apple submit APP_ID --version "2.1.0"
```

### Publish to Google Play

```bash
st google builds upload --app-id com.example.app --file app-release.aab
st google tracks update --app-id com.example.app --track production \
  --version-code 42 --rollout-fraction 0.1
st google listings update --app-id com.example.app --locale en-US \
  --title "My App" --short-description "Short." --full-description "Full."
st google submit --app-id com.example.app
```

### Upload Screenshots (Apple)

```bash
st apple screenshots sets list --version-id VER_ID --locale en-US
st apple screenshots sets create --version-id VER_ID --locale en-US \
  --display-type APP_IPHONE_67
st apple screenshots images upload --set-id SET_ID --file screenshot.png
```

### Upload Images (Google)

```bash
st google images upload --app-id com.example.app --locale en-US \
  --image-type phoneScreenshots --file screenshot.png
```

### Manage TestFlight

```bash
st apple testflight groups list --app-id APP_ID
st apple testflight testers add --group-id GROUP_ID \
  --email tester@example.com --first-name Jane --last-name Doe
```

### In-App Purchases

```bash
st apple iap list --app-id APP_ID
st apple iap create --app-id APP_ID --product-id com.example.coins100 \
  --type consumable --reference-name "100 Coins"
st apple iap localizations create --iap-id IAP_ID --locale en-US \
  --display-name "100 Coins" --description "Buy 100 coins"
st apple iap submit --iap-id IAP_ID
```

### Reviews

```bash
st apple reviews list --app-id APP_ID --limit 20
st apple reviews respond --review-id REV_ID --response "Thanks!"

st google reviews list --app-id com.example.app
st google reviews reply --review-id REV_ID --reply "Thank you!"
```

### Pricing and Availability

```bash
st apple pricing get --app-id APP_ID
st apple pricing set --app-id APP_ID --price-point POINT_ID
st apple availability set --app-id APP_ID --territories US,GB,DE,JP
```

### Phased Release

```bash
st apple phased-release create --version-id VER_ID
st apple phased-release update --version-id VER_ID --state PAUSE
st apple phased-release delete --version-id VER_ID
```

## Error Handling

When a command fails:
1. Check the exit code (2 = auth, 3 = API, 4 = network).
2. Parse the JSON error from stderr: `st ... 2>&1 | jq '.error'`
3. For auth errors, run `st auth status` and re-login if needed.
4. For rate limits (HTTP 429), wait and retry.

## Tips for Agents

- Always use `--output json` (the default) and parse with `jq`.
- Use `--paginate` when you need complete lists.
- Chain commands by extracting IDs with `jq -r '.data[].id'`.
- Loop over locales for multi-language updates rather than making separate requests to the user.
- Verify state after mutations: after `submit`, check versions list to confirm status.
- Disable background update checks in CI with `STOREOPS_NO_UPDATE_CHECK=1`.
