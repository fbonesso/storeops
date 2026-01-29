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

### `st auth login`
Authenticate with a store.

| Flag | Required | Description |
|------|----------|-------------|
| `--store apple\|google` | Yes | Target store |
| `--key-id <id>` | Apple | App Store Connect API key ID |
| `--issuer-id <id>` | Apple | Issuer ID from App Store Connect |
| `--key-path <path>` | Apple | Path to .p8 private key file |
| `--service-account <path>` | Google | Path to service account JSON |
| `--name <profile>` | No | Save as a named profile |

### `st auth switch <profile>`
Switch active profile.

### `st auth status`
Show current authentication state, active profile, and token expiry.

### `st auth init`
Interactive-free initialization: reads credentials from environment variables (`STOREOPS_APPLE_KEY_ID`, `STOREOPS_APPLE_ISSUER_ID`, `STOREOPS_APPLE_KEY_PATH`, `STOREOPS_GOOGLE_SERVICE_ACCOUNT`).

---

## apple apps

### `st apple apps list`
List all apps in the account.

### `st apple apps info --app-id <id>`
Get details for a specific app.

---

## apple versions

### `st apple versions list --app-id <id>`
List versions. Supports `--limit`, `--platform ios|macos|tvos`.

### `st apple versions create --app-id <id> --version <v> --platform <p>`
Create a new app store version.

### `st apple versions update --version-id <id> [--build-id <id>]`
Update a version (e.g., attach a build).

---

## apple builds

### `st apple builds list --app-id <id>`
List processed builds. Supports `--limit`.

### `st apple builds info --build-id <id>`
Get build details including processing state.

---

## apple testflight

### `st apple testflight groups list --app-id <id>`
List beta groups.

### `st apple testflight groups create --app-id <id> --name <name>`
Create a beta group. Optional: `--public`, `--feedback-enabled`.

### `st apple testflight testers list --group-id <id>`
List testers in a group.

### `st apple testflight testers add --group-id <id> --email <email>`
Add a tester. Optional: `--first-name`, `--last-name`.

---

## apple submit

### `st apple submit <app-id> --version <v>`
Submit a version for App Review.

---

## apple reviews

### `st apple reviews list --app-id <id>`
List customer reviews. Supports `--limit`, `--sort`, `--rating`.

### `st apple reviews respond --review-id <id> --response <text>`
Respond to a review.

---

## apple devices

### `st apple devices list`
List registered devices. Supports `--platform`.

### `st apple devices register --name <name> --udid <udid> --platform <p>`
Register a device.

---

## apple analytics

### `st apple analytics sales --app-id <id>`
Fetch sales and trends data. Supports `--frequency daily|weekly|monthly`, `--start-date`, `--end-date`.

---

## apple metadata

### Localizations

| Command | Key Flags |
|---------|-----------|
| `st apple metadata localizations list --version-id <id>` | `--locale` (optional filter) |
| `st apple metadata localizations get --localization-id <id>` | |
| `st apple metadata localizations create --version-id <id> --locale <l>` | `--description`, `--keywords`, `--whats-new`, `--marketing-url`, `--support-url`, `--promotional-text` |
| `st apple metadata localizations update --localization-id <id>` | Same optional fields as create |
| `st apple metadata localizations delete --localization-id <id>` | |

### App Info

| Command | Key Flags |
|---------|-----------|
| `st apple metadata app-info list --app-id <id>` | |
| `st apple metadata app-info create --app-id <id>` | `--locale`, metadata fields |
| `st apple metadata app-info update --info-id <id>` | metadata fields |
| `st apple metadata app-info delete --info-id <id>` | |

### Categories

| Command | Key Flags |
|---------|-----------|
| `st apple metadata categories list` | |
| `st apple metadata categories get --category-id <id>` | |
| `st apple metadata categories set --app-id <id>` | `--primary <id>`, `--secondary <id>` |

---

## apple screenshots

### Sets

| Command | Key Flags |
|---------|-----------|
| `st apple screenshots sets list --version-id <id>` | `--locale` |
| `st apple screenshots sets create --version-id <id> --locale <l> --display-type <t>` | Display types: `APP_IPHONE_67`, `APP_IPHONE_65`, `APP_IPAD_PRO_129`, etc. |
| `st apple screenshots sets delete --set-id <id>` | |

### Images

| Command | Key Flags |
|---------|-----------|
| `st apple screenshots images list --set-id <id>` | |
| `st apple screenshots images upload --set-id <id> --file <path>` | |
| `st apple screenshots images delete --image-id <id>` | |
| `st apple screenshots images reorder --set-id <id> --image-ids <json-array>` | |

---

## apple previews

### Sets

| Command | Key Flags |
|---------|-----------|
| `st apple previews sets list --version-id <id>` | `--locale` |
| `st apple previews sets create --version-id <id> --locale <l> --preview-type <t>` | |
| `st apple previews sets delete --set-id <id>` | |

### Videos

| Command | Key Flags |
|---------|-----------|
| `st apple previews videos list --set-id <id>` | |
| `st apple previews videos upload --set-id <id> --file <path>` | |
| `st apple previews videos delete --video-id <id>` | |

---

## apple pricing

| Command | Key Flags |
|---------|-----------|
| `st apple pricing get --app-id <id>` | |
| `st apple pricing points --app-id <id>` | `--territory` |
| `st apple pricing set --app-id <id> --price-point <id>` | |

---

## apple age-rating

| Command | Key Flags |
|---------|-----------|
| `st apple age-rating get --app-id <id>` | |
| `st apple age-rating update --app-id <id>` | `--alcohol-tobacco-drugs NONE\|INFREQUENT\|FREQUENT`, `--gambling <bool>`, `--violence NONE\|INFREQUENT\|FREQUENT\|GRAPHIC`, etc. |

---

## apple phased-release

| Command | Key Flags |
|---------|-----------|
| `st apple phased-release get --version-id <id>` | |
| `st apple phased-release create --version-id <id>` | |
| `st apple phased-release update --version-id <id>` | `--state ACTIVE\|PAUSE\|COMPLETE` |
| `st apple phased-release delete --version-id <id>` | |

---

## apple iap (In-App Purchases)

| Command | Key Flags |
|---------|-----------|
| `st apple iap list --app-id <id>` | `--type consumable\|non-consumable\|non-renewing` |
| `st apple iap get --iap-id <id>` | |
| `st apple iap create --app-id <id> --product-id <pid> --type <t> --reference-name <n>` | |
| `st apple iap update --iap-id <id>` | `--reference-name`, `--review-note`, `--cleared-for-sale` |
| `st apple iap delete --iap-id <id>` | |
| `st apple iap localizations create --iap-id <id> --locale <l> --display-name <n> --description <d>` | |
| `st apple iap prices set --iap-id <id> --price-point <pid>` | `--territory` |
| `st apple iap submit --iap-id <id>` | |

---

## apple subscriptions

| Command | Key Flags |
|---------|-----------|
| `st apple subscriptions groups list --app-id <id>` | |
| `st apple subscriptions groups create --app-id <id> --reference-name <n>` | |
| `st apple subscriptions items list --group-id <id>` | |
| `st apple subscriptions items create --group-id <id> --product-id <pid> --reference-name <n>` | `--duration <d>` |
| `st apple subscriptions localizations create --subscription-id <id> --locale <l>` | `--display-name`, `--description` |
| `st apple subscriptions prices set --subscription-id <id> --price-point <pid>` | `--territory`, `--preserve-existing` |
| `st apple subscriptions offers create --subscription-id <id>` | `--type introductory\|promotional\|offer-code`, `--duration`, `--mode pay-as-you-go\|pay-up-front\|free` |

---

## apple availability

| Command | Key Flags |
|---------|-----------|
| `st apple availability get --app-id <id>` | |
| `st apple availability territories --app-id <id>` | |
| `st apple availability set --app-id <id> --territories <csv>` | |

---

## google apps

### `st google apps list`
### `st google apps info --app-id <package-name>`

---

## google tracks

### `st google tracks list --app-id <id>`
List tracks (internal, alpha, beta, production).

### `st google tracks update --app-id <id> --track <name>`
Update a track. Flags: `--version-code <n>`, `--rollout-fraction <0.0-1.0>`, `--release-notes <json>`.

---

## google builds

### `st google builds list --app-id <id>`
### `st google builds upload --app-id <id> --file <path>`
Upload an AAB or APK.

---

## google testers

### `st google testers list --app-id <id> --track <name>`
### `st google testers add --app-id <id> --track <name> --email <email>`

---

## google submit

### `st google submit --app-id <id>`
Commit the current edit (finalizes all pending changes).

---

## google reviews

### `st google reviews list --app-id <id>`
### `st google reviews reply --review-id <id> --reply <text>`

---

## google reports

### `st google reports stats --app-id <id>`
Flags: `--metric installs\|crashes\|ratings\|revenue`, `--start-date`, `--end-date`.

---

## google listings

| Command | Key Flags |
|---------|-----------|
| `st google listings list --app-id <id>` | |
| `st google listings get --app-id <id> --locale <l>` | |
| `st google listings update --app-id <id> --locale <l>` | `--title`, `--short-description`, `--full-description` |
| `st google listings delete --app-id <id> --locale <l>` | |

---

## google images

| Command | Key Flags |
|---------|-----------|
| `st google images list --app-id <id> --locale <l> --image-type <t>` | Types: `phoneScreenshots`, `sevenInchScreenshots`, `tenInchScreenshots`, `tvScreenshots`, `wearScreenshots`, `icon`, `featureGraphic`, `tvBanner` |
| `st google images upload --app-id <id> --locale <l> --image-type <t> --file <path>` | |
| `st google images delete --app-id <id> --locale <l> --image-type <t> --image-id <id>` | |
| `st google images delete-all --app-id <id> --locale <l> --image-type <t>` | |

---

## google inapp

### Products

| Command | Key Flags |
|---------|-----------|
| `st google inapp products list --app-id <id>` | |
| `st google inapp products get --app-id <id> --sku <sku>` | |
| `st google inapp products create --app-id <id> --sku <sku>` | `--default-price <micros>`, `--currency <c>`, `--title <t>`, `--description <d>` |
| `st google inapp products update --app-id <id> --sku <sku>` | Same as create |
| `st google inapp products delete --app-id <id> --sku <sku>` | |

### Subscriptions

| Command | Key Flags |
|---------|-----------|
| `st google inapp subscriptions list --app-id <id>` | |
| `st google inapp subscriptions get --app-id <id> --sku <sku>` | |
| `st google inapp subscriptions create --app-id <id> --sku <sku>` | `--period`, `--title`, `--description`, `--default-price`, `--currency` |
| `st google inapp subscriptions archive --app-id <id> --sku <sku>` | |

---

## google availability

| Command | Key Flags |
|---------|-----------|
| `st google availability get --app-id <id>` | |
| `st google availability countries --app-id <id>` | |
| `st google availability update --app-id <id> --countries <csv>` | |
