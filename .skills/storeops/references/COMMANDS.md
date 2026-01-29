# StoreOps Command Reference

## Global Flags

| Flag | Description |
|------|-------------|
| `--output json\|table\|markdown` | Output format (default: json) |
| `--pretty` | Pretty-print JSON output |
| `--profile <name>` | Use a named auth profile |
| `--limit <n>` | Max items per page |
| `--next <cursor>` | Pagination cursor for next page |
| `--paginate` | Auto-fetch all pages |
| `--timeout <ms>` | Request timeout in milliseconds |
| `--verbose` | Enable verbose logging to stderr |

---

## auth

### `storeops auth login`
Authenticate with a store.

| Flag | Required | Description |
|------|----------|-------------|
| `--store apple\|google` | Yes | Target store |
| `--key-id <id>` | Apple | App Store Connect API key ID |
| `--issuer-id <id>` | Apple | Issuer ID from App Store Connect |
| `--key-path <path>` | Apple | Path to .p8 private key file |
| `--service-account <path>` | Google | Path to service account JSON |
| `--name <profile>` | No | Save as a named profile |

### `storeops auth switch <profile>`
Switch active profile.

### `storeops auth status`
Show current authentication state, active profile, and token expiry.

### `storeops auth init`
Interactive-free initialization: reads credentials from environment variables (`STOREOPS_APPLE_KEY_ID`, `STOREOPS_APPLE_ISSUER_ID`, `STOREOPS_APPLE_KEY_PATH`, `STOREOPS_GOOGLE_SERVICE_ACCOUNT`).

---

## apple apps

### `storeops apple apps list`
List all apps in the account.

### `storeops apple apps info --app-id <id>`
Get details for a specific app.

---

## apple versions

### `storeops apple versions list --app-id <id>`
List versions. Supports `--limit`, `--platform ios|macos|tvos`.

### `storeops apple versions create --app-id <id> --version <v> --platform <p>`
Create a new app store version.

### `storeops apple versions update --version-id <id> [--build-id <id>]`
Update a version (e.g., attach a build).

---

## apple builds

### `storeops apple builds list --app-id <id>`
List processed builds. Supports `--limit`.

### `storeops apple builds info --build-id <id>`
Get build details including processing state.

---

## apple testflight

### `storeops apple testflight groups list --app-id <id>`
List beta groups.

### `storeops apple testflight groups create --app-id <id> --name <name>`
Create a beta group. Optional: `--public`, `--feedback-enabled`.

### `storeops apple testflight testers list --group-id <id>`
List testers in a group.

### `storeops apple testflight testers add --group-id <id> --email <email>`
Add a tester. Optional: `--first-name`, `--last-name`.

---

## apple submit

### `storeops apple submit <app-id> --version <v>`
Submit a version for App Review.

---

## apple reviews

### `storeops apple reviews list --app-id <id>`
List customer reviews. Supports `--limit`, `--sort`, `--rating`.

### `storeops apple reviews respond --review-id <id> --response <text>`
Respond to a review.

---

## apple devices

### `storeops apple devices list`
List registered devices. Supports `--platform`.

### `storeops apple devices register --name <name> --udid <udid> --platform <p>`
Register a device.

---

## apple analytics

### `storeops apple analytics sales --app-id <id>`
Fetch sales and trends data. Supports `--frequency daily|weekly|monthly`, `--start-date`, `--end-date`.

---

## apple metadata

### Localizations

| Command | Key Flags |
|---------|-----------|
| `storeops apple metadata localizations list --version-id <id>` | `--locale` (optional filter) |
| `storeops apple metadata localizations get --localization-id <id>` | |
| `storeops apple metadata localizations create --version-id <id> --locale <l>` | `--description`, `--keywords`, `--whats-new`, `--marketing-url`, `--support-url`, `--promotional-text` |
| `storeops apple metadata localizations update --localization-id <id>` | Same optional fields as create |
| `storeops apple metadata localizations delete --localization-id <id>` | |

### App Info

| Command | Key Flags |
|---------|-----------|
| `storeops apple metadata app-info list --app-id <id>` | |
| `storeops apple metadata app-info create --app-id <id>` | `--locale`, metadata fields |
| `storeops apple metadata app-info update --info-id <id>` | metadata fields |
| `storeops apple metadata app-info delete --info-id <id>` | |

### Categories

| Command | Key Flags |
|---------|-----------|
| `storeops apple metadata categories list` | |
| `storeops apple metadata categories get --category-id <id>` | |
| `storeops apple metadata categories set --app-id <id>` | `--primary <id>`, `--secondary <id>` |

---

## apple screenshots

### Sets

| Command | Key Flags |
|---------|-----------|
| `storeops apple screenshots sets list --version-id <id>` | `--locale` |
| `storeops apple screenshots sets create --version-id <id> --locale <l> --display-type <t>` | Display types: `APP_IPHONE_67`, `APP_IPHONE_65`, `APP_IPAD_PRO_129`, etc. |
| `storeops apple screenshots sets delete --set-id <id>` | |

### Images

| Command | Key Flags |
|---------|-----------|
| `storeops apple screenshots images list --set-id <id>` | |
| `storeops apple screenshots images upload --set-id <id> --file <path>` | |
| `storeops apple screenshots images delete --image-id <id>` | |
| `storeops apple screenshots images reorder --set-id <id> --image-ids <json-array>` | |

---

## apple previews

### Sets

| Command | Key Flags |
|---------|-----------|
| `storeops apple previews sets list --version-id <id>` | `--locale` |
| `storeops apple previews sets create --version-id <id> --locale <l> --preview-type <t>` | |
| `storeops apple previews sets delete --set-id <id>` | |

### Videos

| Command | Key Flags |
|---------|-----------|
| `storeops apple previews videos list --set-id <id>` | |
| `storeops apple previews videos upload --set-id <id> --file <path>` | |
| `storeops apple previews videos delete --video-id <id>` | |

---

## apple pricing

| Command | Key Flags |
|---------|-----------|
| `storeops apple pricing get --app-id <id>` | |
| `storeops apple pricing points --app-id <id>` | `--territory` |
| `storeops apple pricing set --app-id <id> --price-point <id>` | |

---

## apple age-rating

| Command | Key Flags |
|---------|-----------|
| `storeops apple age-rating get --app-id <id>` | |
| `storeops apple age-rating update --app-id <id>` | `--alcohol-tobacco-drugs NONE\|INFREQUENT\|FREQUENT`, `--gambling <bool>`, `--violence NONE\|INFREQUENT\|FREQUENT\|GRAPHIC`, etc. |

---

## apple phased-release

| Command | Key Flags |
|---------|-----------|
| `storeops apple phased-release get --version-id <id>` | |
| `storeops apple phased-release create --version-id <id>` | |
| `storeops apple phased-release update --version-id <id>` | `--state ACTIVE\|PAUSE\|COMPLETE` |
| `storeops apple phased-release delete --version-id <id>` | |

---

## apple iap (In-App Purchases)

| Command | Key Flags |
|---------|-----------|
| `storeops apple iap list --app-id <id>` | `--type consumable\|non-consumable\|non-renewing` |
| `storeops apple iap get --iap-id <id>` | |
| `storeops apple iap create --app-id <id> --product-id <pid> --type <t> --reference-name <n>` | |
| `storeops apple iap update --iap-id <id>` | `--reference-name`, `--review-note`, `--cleared-for-sale` |
| `storeops apple iap delete --iap-id <id>` | |
| `storeops apple iap localizations create --iap-id <id> --locale <l> --display-name <n> --description <d>` | |
| `storeops apple iap prices set --iap-id <id> --price-point <pid>` | `--territory` |
| `storeops apple iap submit --iap-id <id>` | |

---

## apple subscriptions

| Command | Key Flags |
|---------|-----------|
| `storeops apple subscriptions groups list --app-id <id>` | |
| `storeops apple subscriptions groups create --app-id <id> --reference-name <n>` | |
| `storeops apple subscriptions items list --group-id <id>` | |
| `storeops apple subscriptions items create --group-id <id> --product-id <pid> --reference-name <n>` | `--duration <d>` |
| `storeops apple subscriptions localizations create --subscription-id <id> --locale <l>` | `--display-name`, `--description` |
| `storeops apple subscriptions prices set --subscription-id <id> --price-point <pid>` | `--territory`, `--preserve-existing` |
| `storeops apple subscriptions offers create --subscription-id <id>` | `--type introductory\|promotional\|offer-code`, `--duration`, `--mode pay-as-you-go\|pay-up-front\|free` |

---

## apple availability

| Command | Key Flags |
|---------|-----------|
| `storeops apple availability get --app-id <id>` | |
| `storeops apple availability territories --app-id <id>` | |
| `storeops apple availability set --app-id <id> --territories <csv>` | |

---

## google apps

### `storeops google apps info <package-name>`
Get app details for a known package name. Note: Google Play API does not provide a list-apps endpoint.

---

## google tracks

### `storeops google tracks list --app-id <id>`
List tracks (internal, alpha, beta, production).

### `storeops google tracks update --app-id <id> --track <name>`
Update a track. Flags: `--version-code <n>`, `--rollout-fraction <0.0-1.0>`, `--release-notes <json>`.

---

## google builds

### `storeops google builds list --app-id <id>`
### `storeops google builds upload --app-id <id> --file <path>`
Upload an AAB or APK.

---

## google testers

### `storeops google testers list --app-id <id> --track <name>`
### `storeops google testers add --app-id <id> --track <name> --email <email>`

---

## google submit

### `storeops google submit --app-id <id>`
Commit the current edit (finalizes all pending changes).

---

## google reviews

### `storeops google reviews list --app-id <id>`
### `storeops google reviews reply --review-id <id> --reply <text>`

---

## google listings

| Command | Key Flags |
|---------|-----------|
| `storeops google listings list --app-id <id>` | |
| `storeops google listings get --app-id <id> --locale <l>` | |
| `storeops google listings update --app-id <id> --locale <l>` | `--title`, `--short-description`, `--full-description` |
| `storeops google listings delete --app-id <id> --locale <l>` | |

---

## google images

| Command | Key Flags |
|---------|-----------|
| `storeops google images list --app-id <id> --locale <l> --image-type <t>` | Types: `phoneScreenshots`, `sevenInchScreenshots`, `tenInchScreenshots`, `tvScreenshots`, `wearScreenshots`, `icon`, `featureGraphic`, `tvBanner` |
| `storeops google images upload --app-id <id> --locale <l> --image-type <t> --file <path>` | |
| `storeops google images delete --app-id <id> --locale <l> --image-type <t> --image-id <id>` | |
| `storeops google images delete-all --app-id <id> --locale <l> --image-type <t>` | |

---

## google inapp

### Products

| Command | Key Flags |
|---------|-----------|
| `storeops google inapp products list --app-id <id>` | |
| `storeops google inapp products get --app-id <id> --sku <sku>` | |
| `storeops google inapp products create --app-id <id> --sku <sku>` | `--default-price <micros>`, `--currency <c>`, `--title <t>`, `--description <d>` |
| `storeops google inapp products update --app-id <id> --sku <sku>` | Same as create |
| `storeops google inapp products delete --app-id <id> --sku <sku>` | |

### Subscriptions

| Command | Key Flags |
|---------|-----------|
| `storeops google inapp subscriptions list --app-id <id>` | |
| `storeops google inapp subscriptions get --app-id <id> --sku <sku>` | |
| `storeops google inapp subscriptions create --app-id <id> --sku <sku>` | `--period`, `--title`, `--description`, `--default-price`, `--currency` |
| `storeops google inapp subscriptions archive --app-id <id> --sku <sku>` | |

---

## google availability

| Command | Key Flags |
|---------|-----------|
| `storeops google availability get <package-name> --track <track>` | |
| `storeops google availability countries <package-name> --track <track>` | |
| `storeops google availability update <package-name> --track <track> --countries <csv>` | `--rest-of-world` |
